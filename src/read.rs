// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ops::Deref;
use std::{char, cmp, io, str};

#[cfg(feature = "raw_value")]
use serde::de::Visitor;

use iter::LineColIterator;

use error::{Error, ErrorCode, Result};

#[cfg(feature = "raw_value")]
use raw::{BorrowedRawDeserializer, OwnedRawDeserializer};
use de::ParseDecision;

/// Trait used by the deserializer for iterating over input. This is manually
/// "specialized" for iterating over &[u8]. Once feature(specialization) is
/// stable we can use actual specialization.
///
/// This trait is sealed and cannot be implemented for types outside of
/// `serde_edn`.
pub trait Read<'de>: private::Sealed {
    #[doc(hidden)]
    fn next(&mut self) -> Result<Option<u8>>;
    #[doc(hidden)]
    fn peek(&mut self) -> Result<Option<u8>>;

    /// Only valid after a call to peek(). Discards the peeked byte.
    #[doc(hidden)]
    fn discard(&mut self);

    /// Position of the most recent call to next().
    ///
    /// The most recent call was probably next() and not peek(), but this method
    /// should try to return a sensible result if the most recent call was
    /// actually peek() because we don't always know.
    ///
    /// Only called in case of an error, so performance is not important.
    #[doc(hidden)]
    fn position(&self) -> Position;

    /// Position of the most recent call to peek().
    ///
    /// The most recent call was probably peek() and not next(), but this method
    /// should try to return a sensible result if the most recent call was
    /// actually next() because we don't always know.
    ///
    /// Only called in case of an error, so performance is not important.
    #[doc(hidden)]
    fn peek_position(&self) -> Position;

    /// Offset from the beginning of the input to the next byte that would be
    /// returned by next() or peek().
    #[doc(hidden)]
    fn byte_offset(&self) -> usize;

    /// Assumes the previous byte was a quotation mark. Parses a edn-escaped
    /// string until the next quotation mark using the given scratch space if
    /// necessary. The scratch space is initially empty.
    #[doc(hidden)]
    fn parse_str<'s>(&'s mut self, scratch: &'s mut Vec<u8>) -> Result<Reference<'de, 's, str>>;

    /// Presumes valid symbol start sequence.
    /// Returns the str until the next whitespace using the given scratch space if
    /// necessary. The scratch space is initially empty.
    #[doc(hidden)]
    fn parse_symbol<'s>(&'s mut self, scratch: &'s mut Vec<u8>) -> Result<Reference<'de, 's, str>>;
    fn parse_symbol_offset<'s>(&'s mut self, scratch: &'s mut Vec<u8>, offset:usize) -> Result<Reference<'de, 's, str>>;

    fn parse_reserved_or_symbol<'s >(
        &'s mut self, scratch: &'s mut Vec<u8>,
        offset: &mut usize,
        reserved_len: usize,
        reserved_bytes: &[u8; 5]) -> Result<ParseDecision>;

    fn parse_keyword<'s>(&'s mut self, scratch: &'s mut Vec<u8>) -> Result<Reference<'de, 's, str>>;

    /// Assumes the previous byte was a quotation mark. Parses a edn-escaped
    /// string until the next quotation mark using the given scratch space if
    /// necessary. The scratch space is initially empty.
    ///
    /// This function returns the raw bytes in the string with escape sequences
    /// expanded but without performing unicode validation.
    #[doc(hidden)]
    fn parse_str_raw<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
    ) -> Result<Reference<'de, 's, [u8]>>;

    /// Assumes the previous byte was a quotation mark. Parses a edn-escaped
    /// string until the next quotation mark but discards the data.
    #[doc(hidden)]
    fn ignore_str(&mut self) -> Result<()>;

    /// Assumes the previous byte was a hex escape sequnce ('\u') in a string.
    /// Parses next hexadecimal sequence.
    #[doc(hidden)]
    fn decode_hex_escape(&mut self) -> Result<u16>;

    /// Switch raw buffering mode on.
    ///
    /// This is used when deserializing `RawValue`.
    #[cfg(feature = "raw_value")]
    #[doc(hidden)]
    fn begin_raw_buffering(&mut self);

    /// Switch raw buffering mode off and provides the raw buffered data to the
    /// given visitor.
    #[cfg(feature = "raw_value")]
    #[doc(hidden)]
    fn end_raw_buffering<V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>;
}

pub struct Position {
    pub line: usize,
    pub column: usize,
}

pub enum Reference<'b, 'c, T: ?Sized + 'static> {
    Borrowed(&'b T),
    Copied(&'c T),
}

impl<'b, 'c, T: ?Sized + 'static> Deref for Reference<'b, 'c, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match *self {
            Reference::Borrowed(b) => b,
            Reference::Copied(c) => c,
        }
    }
}

/// edn input source that reads from a std::io input stream.
pub struct IoRead<R>
where
    R: io::Read,
{
    iter: LineColIterator<io::Bytes<R>>,
    /// Temporary storage of peeked byte.
    ch: Option<u8>,
    #[cfg(feature = "raw_value")]
    raw_buffer: Option<Vec<u8>>,
}

/// edn input source that reads from a slice of bytes.
//
// This is more efficient than other iterators because peek() can be read-only
// and we can compute line/col position only if an error happens.
pub struct SliceRead<'a> {
    slice: &'a [u8],
    /// Index of the *next* byte that will be returned by next() or peek().
    index: usize,
    #[cfg(feature = "raw_value")]
    raw_buffering_start_index: usize,
}

/// edn input source that reads from a UTF-8 string.
//
// Able to elide UTF-8 checks by assuming that the input is valid UTF-8.
pub struct StrRead<'a> {
    delegate: SliceRead<'a>,
    #[cfg(feature = "raw_value")]
    data: &'a str,
}

// Prevent users from implementing the Read trait.
mod private {
    pub trait Sealed {}
}

//////////////////////////////////////////////////////////////////////////////

impl<R> IoRead<R>
where
    R: io::Read,
{
    /// Create a edn input source to read from a std::io input stream.
    pub fn new(reader: R) -> Self {
        #[cfg(not(feature = "raw_value"))]
        {
            IoRead {
                iter: LineColIterator::new(reader.bytes()),
                ch: None,
            }
        }
        #[cfg(feature = "raw_value")]
        {
            IoRead {
                iter: LineColIterator::new(reader.bytes()),
                ch: None,
                raw_buffer: None,
            }
        }
    }
}

impl<R> private::Sealed for IoRead<R> where R: io::Read {}

impl<R> IoRead<R>
where
    R: io::Read,
{
    fn parse_str_bytes<'s, T, F>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
        validate: bool,
        result: F,
    ) -> Result<T>
    where
        T: 's,
        F: FnOnce(&'s Self, &'s [u8]) -> Result<T>,
    {
        loop {
            let ch = try!(next_or_eof(self));
            if !ESCAPE[ch as usize] {
                scratch.push(ch);
                continue;
            }
            match ch {
                b'"' => {
                    return result(self, scratch);
                }
                b'\\' => {
                    try!(parse_escape(self, scratch));
                }
                _ => {
                    if validate {
                        return error(self, ErrorCode::ControlCharacterWhileParsingString);
                    }
                    scratch.push(ch);
                }
            }
        }
    }

    fn parse_symbol_bytes<'s, T, F>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
        validate: bool,
        result: F,
    ) -> Result<T>
        where
            T: 's,
            F: FnOnce(&'s Self, &'s [u8]) -> Result<T>,
    {
        loop {
            match try!(self.peek()) {
                Some(ch) => {
                    if VALID_SYMBOL_BYTE[ch as usize] {
                        self.discard();
                        scratch.push(ch);
                        continue;
                    }
                    match ch {
                        b')' | b']' | b'}' | b'(' | b'[' | b'{' |
                        b' ' | b'\n' | b'\r' | b'\t' | b',' => {
                            return result(self, scratch);
                        }

                        _ => {
                            // todo. ErrorCode::InvalidSymbol (though this will be called by keyword parse fns)
                            return error(self, ErrorCode::InvalidKeyword);
                        }
                    }
                }
                None => return result(self, scratch)
            }
        }
    }
}

impl<'de, R> Read<'de> for IoRead<R>
where
    R: io::Read,
{
    #[inline]
    fn next(&mut self) -> Result<Option<u8>> {
        match self.ch.take() {
            Some(ch) => {
                #[cfg(feature = "raw_value")]
                {
                    if let Some(ref mut buf) = self.raw_buffer {
                        buf.push(ch);
                    }
                }
                Ok(Some(ch))
            }
            None => match self.iter.next() {
                Some(Err(err)) => Err(Error::io(err)),
                Some(Ok(ch)) => {
                    #[cfg(feature = "raw_value")]
                    {
                        if let Some(ref mut buf) = self.raw_buffer {
                            buf.push(ch);
                        }
                    }
                    Ok(Some(ch))
                }
                None => Ok(None),
            },
        }
    }

    #[inline]
    fn peek(&mut self) -> Result<Option<u8>> {
        match self.ch {
            Some(ch) => Ok(Some(ch)),
            None => match self.iter.next() {
                Some(Err(err)) => Err(Error::io(err)),
                Some(Ok(ch)) => {
                    self.ch = Some(ch);
                    Ok(self.ch)
                }
                None => Ok(None),
            },
        }
    }

    #[cfg(not(feature = "raw_value"))]
    #[inline]
    fn discard(&mut self) {
        self.ch = None;
    }

    #[cfg(feature = "raw_value")]
    fn discard(&mut self) {
        if let Some(ch) = self.ch.take() {
            if let Some(ref mut buf) = self.raw_buffer {
                buf.push(ch);
            }
        }
    }

    fn position(&self) -> Position {
        Position {
            line: self.iter.line(),
            column: self.iter.col(),
        }
    }

    fn peek_position(&self) -> Position {
        // The LineColIterator updates its position during peek() so it has the
        // right one here.
        self.position()
    }

    fn byte_offset(&self) -> usize {
        match self.ch {
            Some(_) => self.iter.byte_offset() - 1,
            None => self.iter.byte_offset(),
        }
    }

    fn parse_symbol<'s>(&'s mut self, scratch: &'s mut Vec<u8>) -> Result<Reference<'de, 's, str>> {
        self.parse_symbol_bytes(scratch, false, as_str)
            .map(Reference::Copied)
    }

    fn parse_keyword<'s>(&'s mut self, scratch: &'s mut Vec<u8>) -> Result<Reference<'de, 's, str>> {
        self.parse_symbol_bytes(scratch, false, as_str)
            .map(Reference::Copied)
    }


    fn parse_symbol_offset<'s>(&'s mut self, scratch: &'s mut Vec<u8>, offset: usize) -> Result<Reference<'de, 's, str>> {
        // starting at an index is irrelevant here because our parse_symbol_bytes method doesn't hard code a start position
        self.parse_symbol_bytes(scratch, false, as_str)
            .map(Reference::Copied)
    }


    fn parse_str<'s>(&'s mut self, scratch: &'s mut Vec<u8>) -> Result<Reference<'de, 's, str>> {
        self.parse_str_bytes(scratch, true, as_str)
            .map(Reference::Copied)
    }

    fn parse_str_raw<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
    ) -> Result<Reference<'de, 's, [u8]>> {
        self.parse_str_bytes(scratch, false, |_, bytes| Ok(bytes))
            .map(Reference::Copied)
    }

    fn ignore_str(&mut self) -> Result<()> {
        loop {
            let ch = try!(next_or_eof(self));
            if !ESCAPE[ch as usize] {
                continue;
            }
            match ch {
                b'"' => {
                    return Ok(());
                }
                b'\\' => {
                    try!(ignore_escape(self));
                }
                _ => {
                    return error(self, ErrorCode::ControlCharacterWhileParsingString);
                }
            }
        }
    }

    fn decode_hex_escape(&mut self) -> Result<u16> {
        let mut n = 0;
        for _ in 0..4 {
            match decode_hex_val(try!(next_or_eof(self))) {
                None => return error(self, ErrorCode::InvalidEscape),
                Some(val) => {
                    n = (n << 4) + val;
                }
            }
        }
        Ok(n)
    }

    #[cfg(feature = "raw_value")]
    fn begin_raw_buffering(&mut self) {
        self.raw_buffer = Some(Vec::new());
    }

    #[cfg(feature = "raw_value")]
    fn end_raw_buffering<V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let raw = self.raw_buffer.take().unwrap();
        let raw = String::from_utf8(raw).unwrap();
        visitor.visit_map(OwnedRawDeserializer {
            raw_value: Some(raw),
        })
    }

    fn parse_reserved_or_symbol(
        &mut self,
        scratch: &mut Vec<u8>,
        offset: &mut usize,
        reserved_len: usize,
        reserved_bytes: &[u8; 5], //can't generify size so hard coded to max of reserved words i.e. `false` and callers will have to pad to 5
    ) -> Result<ParseDecision> {
        loop {
            match try!(self.next()) {
                Some(b' ') | Some(b'\n') | Some(b'\t') | Some(b'\r') | Some(b',') => {
                    scratch.extend_from_slice(&reserved_bytes[0..*offset]);
                    break Ok(ParseDecision::Symbol);
                }
                Some(v) => {
                    if v == reserved_bytes[*offset] {
                        *offset += 1;

                        if *offset == reserved_len {
                            match try!(self.peek()) {
                                Some(b' ') | Some(b'\n') | Some(b'\t') | Some(b'\r') | Some(b',')
                                | Some(b'"')
                                | Some(b'(') | Some(b'{') | Some(b'{')
                                | Some(b')') | Some(b']') | Some(b'}') => {
                                    break Ok(ParseDecision::Reserved);
                                }
                                Some(v2) => {
                                    scratch.extend_from_slice(&reserved_bytes[0..*offset]);
                                    break Ok(ParseDecision::Symbol);
                                }
                                // eof
                                None => {
                                    break Ok(ParseDecision::Reserved);
                                }
                            }
                        }
                        // loop again because within reserved sequence but not at the end
                        continue;
                    }

                    // not a reserved word but matches the reserved word sequence
                    // up until offset
//                    *offset += 1;
                    scratch.extend_from_slice(&reserved_bytes[0..*offset]);
                    scratch.extend_from_slice(&[v]);
                    break Ok(ParseDecision::Symbol);
                }

                // eof
                None => {
                    if *offset == reserved_len {
                        break Ok(ParseDecision::Reserved);
                    }
                    // not a reserved thing but matches the reserved word sequence
                    // up until offset
                    scratch.extend_from_slice(&reserved_bytes[0..*offset]);
                    break Ok(ParseDecision::Symbol);
                }
            }
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<'a> SliceRead<'a> {
    /// Create a edn input source to read from a slice of bytes.
    pub fn new(slice: &'a [u8]) -> Self {
        #[cfg(not(feature = "raw_value"))]
        {
            SliceRead {
                slice: slice,
                index: 0,
            }
        }
        #[cfg(feature = "raw_value")]
        {
            SliceRead {
                slice: slice,
                index: 0,
                raw_buffering_start_index: 0,
            }
        }
    }

    fn position_of_index(&self, i: usize) -> Position {
        let mut position = Position { line: 1, column: 0 };
        for ch in &self.slice[..i] {
            match *ch {
                b'\n' => {
                    position.line += 1;
                    position.column = 0;
                }
                _ => {
                    position.column += 1;
                }
            }
        }
        position
    }

    fn parse_reserved_or_symbol<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
        offset: &mut usize,
        reserved_len:usize,
        reserved_bytes:&[u8;5], //can't generify size so hard coded to max of reserved words i.e. `false` and callers will have to pad to 5
    ) -> Result<ParseDecision> // this makes me sad but might be better design to separate parse and visit anyhow
       {
        loop {
            match try!(self.next()) {
                Some(b' ') | Some(b'\n') | Some(b'\t') | Some(b'\r') | Some(b',') => {
                    scratch.extend_from_slice(&reserved_bytes[0..*offset]);
                    break Ok(ParseDecision::Symbol)
                }
                Some(v) => {
                    if v == reserved_bytes[*offset] {
                        *offset += 1;

                        if *offset == reserved_len {
                            match try!(self.peek()) {
                                Some(b' ') | Some(b'\n') | Some(b'\t') | Some(b'\r') | Some(b',')
                                | Some(b'"')
                                | Some(b'(') | Some(b'{') | Some(b'{')
                                | Some(b')') | Some(b']') | Some(b'}') => {
                                    break Ok(ParseDecision::Reserved)
                                }
                                Some(v2) => {
                                    scratch.extend_from_slice(&reserved_bytes[0..*offset]);
                                    break Ok(ParseDecision::Symbol)
                                }
                                // eof
                                None => {
                                    break Ok(ParseDecision::Reserved)
                                }
                            }
                        }
                        // loop again because within reserved sequence but not at the end
                        continue;
                    }

                    // not a reserved word but matches the reserved word sequence
                    // up until offset
                    *offset += 1;
                    scratch.extend_from_slice(&reserved_bytes[0..*offset]);
                    scratch.extend_from_slice(&[v]);
                    break Ok(ParseDecision::Symbol)
                }

                // eof
                None => {
                    if *offset == reserved_len {
                        break Ok(ParseDecision::Reserved)
                    }
                    // not a reserved thing but matches the reserved word sequence
                    // up until offset
                    scratch.extend_from_slice(&reserved_bytes[0..*offset]);
                    break Ok(ParseDecision::Symbol)
                }
            }
        }
    }

    fn parse_symbol_bytes_offset<'s, T: ?Sized, F>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
        validate: bool,
        offset:usize,
        result: F,
    ) -> Result<Reference<'a, 's, T>>
        where
            T: 's,
            F: for<'f> FnOnce(&'s Self, &'f [u8]) -> Result<&'f T>,
    {
        // Index of the first byte not yet copied into the scratch space.
        println!("index {}",self.index);
        println!("offset {}",offset);
        println!("scratch {:?}",scratch);
        println!("slice {:?}",self.slice);
        scratch.clear();
        let mut start = self.index-offset;

        loop {
            while self.index < self.slice.len() && VALID_SYMBOL_BYTE[self.slice[self.index] as usize] {
                self.index += 1;
            }
            // symbol or keyword can terminate in EOF or whitespace or `)` `]` `}`
            if self.index == self.slice.len() {
//                return error(self, ErrorCode::EofWhileParsingString);
                // Fast path: return a slice of the raw edn without any
                // copying.
//                let borrowed = &self.slice[start..self.index];
                let borrowed = &self.slice[start..self.index];
                self.index += 1;
                return result(self, borrowed).map(Reference::Borrowed);
            }
            match self.slice[self.index] {
                // done if seq start or terminate
                b')' | b']' | b'}' | b'(' | b'[' | b'{' | b'"' => {
                    if scratch.is_empty() {
                        // Fast path: return a slice of the raw edn without any
                        // copying.
                        let borrowed = &self.slice[start..self.index];
//                        self.index += 1;
//                        println!("got at seq term {:?}",borrowed);
                        return result(self, borrowed).map(Reference::Borrowed);
                    } else {
                        //  todo. expect scratch to be empty always because we don't deal with escape sequences,
                        // remove the check once this appears to be the case
                        unreachable!();
                    }
                }
                // did we iterate until whitespace?
                b' ' | b'\n' | b'\r' | b'\t' |  b',' => {
                    if scratch.is_empty() {
                        // Fast path: return a slice of the raw edn without any
                        // copying.
                        let borrowed = &self.slice[start..self.index];
//                        println!("got at whitespace {:?}",borrowed);
                        self.index += 1;
                        return result(self, borrowed).map(Reference::Borrowed);
                    } else {
                        //  todo. expect scratch to be empty always because we don't deal with escape sequences,
                        // remove the check once this appears to be the case
                        unreachable!();
                    }
                }
                // iterated until invalid symbol character
                c => {
                    println!("fallthrough {:?}",c);
                    // todo. invalid symbol, though keyword uses this also
                    return error(self, ErrorCode::InvalidKeyword)
                }
            }
        }
    }


    /// The big optimization here over IoRead is that if the string contains no
    /// backslash escape sequences, the returned &str is a slice of the raw edn
    /// data so we avoid copying into the scratch space.
    fn parse_symbol_bytes<'s, T: ?Sized, F>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
        validate: bool,
        result: F,
    ) -> Result<Reference<'a, 's, T>>
        where
            T: 's,
            F: for<'f> FnOnce(&'s Self, &'f [u8]) -> Result<&'f T>,
    {
        // Index of the first byte not yet copied into the scratch space.
        let mut start = self.index;

        loop {
            while self.index < self.slice.len() && VALID_SYMBOL_BYTE[self.slice[self.index] as usize] {
                self.index += 1;
            }
            // symbol or keyword can terminate in EOF or whitespace
            if self.index == self.slice.len() {
//                return error(self, ErrorCode::EofWhileParsingString);
                // Fast path: return a slice of the raw edn without any
                // copying.
                let borrowed = &self.slice[start..self.index];
                self.index += 1;
                return result(self, borrowed).map(Reference::Borrowed);
            }
            match self.slice[self.index] {
                // done if seq/str start or terminate
                b')' | b']' | b'}' | b'(' | b'[' | b'{' | b'"' => {
                    if scratch.is_empty() {
                        // Fast path: return a slice of the raw edn without any
                        // copying.
                        let borrowed = &self.slice[start..self.index];
                        // don't move the cursor
//                        self.index += 1;
                        return result(self, borrowed).map(Reference::Borrowed);
                    } else {
                        //  todo. expect scratch to be empty always because we don't deal with escape sequences,
                        // remove the check once this appears to be the case
                        unreachable!();
                    }
                }
                // did we iterate until whitespace?
                b' ' | b'\n' | b'\r' | b'\t' |  b',' => {
                    if scratch.is_empty() {
                        // Fast path: return a slice of the raw edn without any
                        // copying.
                        let borrowed = &self.slice[start..self.index];
//                        self.index += 1; //leave the  whitespace for map delineation
                        return result(self, borrowed).map(Reference::Borrowed);
                    } else {
                        //  todo. expect scratch to be empty always because we don't deal with escape sequences,
                        // remove the check once this appears to be the case
                        unreachable!();
                    }
                }
                // iterated until invalid symbol character
                _ => {
                    println!("fallthrough parse symbol bytes");
                    // todo. invalid symbol
                    return error(self, ErrorCode::InvalidKeyword)
                }
            }
        }
    }

    /// The big optimization here over IoRead is that if the string contains no
    /// backslash escape sequences, the returned &str is a slice of the raw edn
    /// data so we avoid copying into the scratch space.
    fn parse_str_bytes<'s, T: ?Sized, F>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
        validate: bool,
        result: F,
    ) -> Result<Reference<'a, 's, T>>
    where
        T: 's,
        F: for<'f> FnOnce(&'s Self, &'f [u8]) -> Result<&'f T>,
    {
        // Index of the first byte not yet copied into the scratch space.
        let mut start = self.index;

        loop {
            while self.index < self.slice.len() && !ESCAPE[self.slice[self.index] as usize] {
                self.index += 1;
            }
            if self.index == self.slice.len() {
                return error(self, ErrorCode::EofWhileParsingString);
            }
            match self.slice[self.index] {
                b'"' => {
                    if scratch.is_empty() {
                        // Fast path: return a slice of the raw edn without any
                        // copying.
                        let borrowed = &self.slice[start..self.index];
                        self.index += 1;
                        return result(self, borrowed).map(Reference::Borrowed);
                    } else {
                        scratch.extend_from_slice(&self.slice[start..self.index]);
                        self.index += 1;
                        return result(self, scratch).map(Reference::Copied);
                    }
                }
                b'\\' => {
                    scratch.extend_from_slice(&self.slice[start..self.index]);
                    self.index += 1;
                    try!(parse_escape(self, scratch));
                    start = self.index;
                }
                _ => {
                    self.index += 1;
                    if validate {
                        return error(self, ErrorCode::ControlCharacterWhileParsingString);
                    }
                }
            }
        }
    }
}

impl<'a> private::Sealed for SliceRead<'a> {}

impl<'a> Read<'a> for SliceRead<'a> {
    #[inline]
    fn next(&mut self) -> Result<Option<u8>> {
        // `Ok(self.slice.get(self.index).map(|ch| { self.index += 1; *ch }))`
        // is about 10% slower.
        Ok(if self.index < self.slice.len() {
            let ch = self.slice[self.index];
            self.index += 1;
            Some(ch)
        } else {
            None
        })
    }

    #[inline]
    fn peek(&mut self) -> Result<Option<u8>> {
        // `Ok(self.slice.get(self.index).map(|ch| *ch))` is about 10% slower
        // for some reason.
        Ok(if self.index < self.slice.len() {
            Some(self.slice[self.index])
        } else {
            None
        })
    }

    #[inline]
    fn discard(&mut self) {
        self.index += 1;
    }

    fn position(&self) -> Position {
        self.position_of_index(self.index)
    }

    fn peek_position(&self) -> Position {
        // Cap it at slice.len() just in case the most recent call was next()
        // and it returned the last byte.
        self.position_of_index(cmp::min(self.slice.len(), self.index + 1))
    }

    fn byte_offset(&self) -> usize {
        self.index
    }

    fn parse_str<'s>(&'s mut self, scratch: &'s mut Vec<u8>) -> Result<Reference<'a, 's, str>> {
        self.parse_str_bytes(scratch, true, as_str)
    }

    fn parse_symbol<'s>(&'s mut self, scratch: &'s mut Vec<u8>) -> Result<Reference<'a, 's, str>> {
        self.parse_symbol_bytes(scratch, true, as_str)
    }

    fn parse_symbol_offset<'s>(&'s mut self, scratch: &'s mut Vec<u8>, offset: usize) -> Result<Reference<'a, 's, str>> {
        self.parse_symbol_bytes_offset(scratch, true, offset, as_str)
    }

    fn parse_reserved_or_symbol<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
        offset: &mut usize,
        reserved_len: usize,
        reserved_bytes: &[u8; 5],
    ) -> Result<ParseDecision> {
        self.parse_reserved_or_symbol(scratch, offset, reserved_len, reserved_bytes)
    }

    fn parse_keyword<'s>(&'s mut self, scratch: &'s mut Vec<u8>) -> Result<Reference<'a, 's, str>> {
        self.parse_symbol_bytes(scratch, true, as_str)
    }

    fn parse_str_raw<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
    ) -> Result<Reference<'a, 's, [u8]>> {
        self.parse_str_bytes(scratch, false, |_, bytes| Ok(bytes))
    }

    fn ignore_str(&mut self) -> Result<()> {
        loop {
            while self.index < self.slice.len() && !ESCAPE[self.slice[self.index] as usize] {
                self.index += 1;
            }
            if self.index == self.slice.len() {
                return error(self, ErrorCode::EofWhileParsingString);
            }
            match self.slice[self.index] {
                b'"' => {
                    self.index += 1;
                    return Ok(());
                }
                b'\\' => {
                    self.index += 1;
                    try!(ignore_escape(self));
                }
                _ => {
                    return error(self, ErrorCode::ControlCharacterWhileParsingString);
                }
            }
        }
    }

    fn decode_hex_escape(&mut self) -> Result<u16> {
        if self.index + 4 > self.slice.len() {
            self.index = self.slice.len();
            return error(self, ErrorCode::EofWhileParsingString);
        }

        let mut n = 0;
        for _ in 0..4 {
            let ch = decode_hex_val(self.slice[self.index]);
            self.index += 1;
            match ch {
                None => return error(self, ErrorCode::InvalidEscape),
                Some(val) => {
                    n = (n << 4) + val;
                }
            }
        }
        Ok(n)
    }

    #[cfg(feature = "raw_value")]
    fn begin_raw_buffering(&mut self) {
        self.raw_buffering_start_index = self.index;
    }

    #[cfg(feature = "raw_value")]
    fn end_raw_buffering<V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'a>,
    {
        let raw = &self.slice[self.raw_buffering_start_index..self.index];
        let raw = str::from_utf8(raw).unwrap();
        visitor.visit_map(BorrowedRawDeserializer {
            raw_value: Some(raw),
        })
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<'a> StrRead<'a> {
    /// Create a edn input source to read from a UTF-8 string.
    pub fn new(s: &'a str) -> Self {
        #[cfg(not(feature = "raw_value"))]
        {
            StrRead {
                delegate: SliceRead::new(s.as_bytes()),
            }
        }
        #[cfg(feature = "raw_value")]
        {
            StrRead {
                delegate: SliceRead::new(s.as_bytes()),
                data: s,
            }
        }
    }
}

impl<'a> private::Sealed for StrRead<'a> {}

impl<'a> Read<'a> for StrRead<'a> {
    #[inline]
    fn next(&mut self) -> Result<Option<u8>> {
        self.delegate.next()
    }

    #[inline]
    fn peek(&mut self) -> Result<Option<u8>> {
        self.delegate.peek()
    }

    #[inline]
    fn discard(&mut self) {
        self.delegate.discard();
    }

    fn position(&self) -> Position {
        self.delegate.position()
    }

    fn peek_position(&self) -> Position {
        self.delegate.peek_position()
    }

    fn byte_offset(&self) -> usize {
        self.delegate.byte_offset()
    }

    fn parse_str<'s>(&'s mut self, scratch: &'s mut Vec<u8>) -> Result<Reference<'a, 's, str>> {
        self.delegate.parse_str_bytes(scratch, true, |_, bytes| {
            // The input is assumed to be valid UTF-8 and the \u-escapes are
            // checked along the way, so don't need to check here.
            Ok(unsafe { str::from_utf8_unchecked(bytes) })
        })
    }

    fn parse_symbol<'s>(&'s mut self, scratch: &'s mut Vec<u8>) -> Result<Reference<'a, 's, str>> {
        self.delegate.parse_symbol_bytes(scratch, true, |_, bytes| {
            // The input is assumed to be valid UTF-8 and the \u-escapes are
            // checked along the way, so don't need to check here.
            // todo.
            Ok(unsafe { str::from_utf8_unchecked(bytes) })
        })
    }

    fn parse_symbol_offset<'s>(&'s mut self, scratch: &'s mut Vec<u8>, offset: usize) -> Result<Reference<'a, 's, str>> {
        self.delegate.parse_symbol_bytes_offset(scratch, true,offset, |_, bytes| {
            // The input is assumed to be valid UTF-8 and the \u-escapes are
            // checked along the way, so don't need to check here.
            // todo.
            Ok(unsafe { str::from_utf8_unchecked(bytes) })
        })
    }

    fn parse_reserved_or_symbol<'s, >(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
        offset: &mut usize,
        reserved_len: usize,
        reserved_bytes: &[u8; 5],
        ) -> Result<ParseDecision> {
        self.delegate.parse_reserved_or_symbol(scratch, offset, reserved_len, reserved_bytes)
    }

    fn parse_keyword<'s>(&'s mut self, scratch: &'s mut Vec<u8>) -> Result<Reference<'a, 's, str>> {
        self.delegate.parse_symbol_bytes(scratch, true, |_, bytes| {
            // The input is assumed to be valid UTF-8 and the \u-escapes are
            // checked along the way, so don't need to check here.
            Ok(unsafe { str::from_utf8_unchecked(bytes) })
        })
    }

    fn parse_str_raw<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
    ) -> Result<Reference<'a, 's, [u8]>> {
        self.delegate.parse_str_raw(scratch)
    }

    fn ignore_str(&mut self) -> Result<()> {
        self.delegate.ignore_str()
    }

    fn decode_hex_escape(&mut self) -> Result<u16> {
        self.delegate.decode_hex_escape()
    }

    #[cfg(feature = "raw_value")]
    fn begin_raw_buffering(&mut self) {
        self.delegate.begin_raw_buffering()
    }

    #[cfg(feature = "raw_value")]
    fn end_raw_buffering<V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'a>,
    {
        let raw = &self.data[self.delegate.raw_buffering_start_index..self.delegate.index];
        visitor.visit_map(BorrowedRawDeserializer {
            raw_value: Some(raw),
        })
    }
}

//////////////////////////////////////////////////////////////////////////////

// Lookup table of bytes that must be escaped. A value of true at index i means
// that byte i requires an escape sequence in the input.
static ESCAPE: [bool; 256] = {
    const CT: bool = true; // control character \x00...\x1F
    const QU: bool = true; // quote \x22
    const BS: bool = true; // backslash \x5C
    const __: bool = false; // allow unescaped
    [
        //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
        CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 0
        CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 1
        __, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
        __, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
    ]
};
// Lookup table of bytes that are allowed in symbols. A value of true at index i means
// that byte i is valid.
// Only for symbol body once start sequence validation complete
// any whitespace is invalid
static VALID_SYMBOL_BYTE: [bool; 256] = {
    // . * + ! - _ ? $ % & = < > [A-Z] [a-z] [0-9]
    const ST: bool = true; //  star \x2A
    const PD: bool = true; //  period \x2E
    const PL: bool = true; //  plus \x2B
    const BG: bool = true; // bang \x21
    const MI: bool = true; // minus \x2D
    const UN: bool = true; // underscore \x5F
    const QM: bool = true; // question mark \x3F
    const DL: bool = true; // dollar sign \x24
    const PC: bool = true; // percent \x25
    const AM: bool = true; // ampersand \x26
    const EQ: bool = true; // equals \x3D
    const LT: bool = true; // less than \x3C
    const GT: bool = true; // greater than \x3E
    const AU: bool = true; // alpha upper \x41 - \x5A
    const AL: bool = true; // alpha lower \x61 - \x7A
    const NU: bool = true; // number \x30 - \x39

    const __ : bool = false; // invalid
    [
        //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 0
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 1
        __, BG, __, __, DL, PC, AM, __, __, __, ST, PL, __, MI, PD, __, // 2
        __, __, __, __, __, __, __, __, __, __, __, __, LT, EQ, GT, QM, // 3
        __, AU, AU, AU, AU, AU, AU, AU, AU, AU, AU, AU, AU, AU, AU, AU, // 4
        AU, AU, AU, AU, AU, AU, AU, AU, AU, AU, AU, __, __, __, __, UN, // 5
        __, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, // 6
        AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, AL, __, __, __, __, __, // 7
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
    ]
};


fn next_or_eof<'de, R: ?Sized + Read<'de>>(read: &mut R) -> Result<u8> {
    match try!(read.next()) {
        Some(b) => Ok(b),
        None => error(read, ErrorCode::EofWhileParsingString),
    }
}

fn error<'de, R: ?Sized + Read<'de>, T>(read: &R, reason: ErrorCode) -> Result<T> {
    let position = read.position();
    Err(Error::syntax(reason, position.line, position.column))
}

fn as_str<'de, 's, R: Read<'de>>(read: &R, slice: &'s [u8]) -> Result<&'s str> {
    str::from_utf8(slice).or_else(|_| error(read, ErrorCode::InvalidUnicodeCodePoint))
}

/// Parses a edn escape sequence and appends it into the scratch space. Assumes
/// the previous byte read was a backslash.
fn parse_escape<'de, R: Read<'de>>(read: &mut R, scratch: &mut Vec<u8>) -> Result<()> {
    let ch = try!(next_or_eof(read));

    match ch {
        b'"' => scratch.push(b'"'),
        b'\\' => scratch.push(b'\\'),
        b'/' => scratch.push(b'/'),
        b'b' => scratch.push(b'\x08'),
        b'f' => scratch.push(b'\x0c'),
        b'n' => scratch.push(b'\n'),
        b'r' => scratch.push(b'\r'),
        b't' => scratch.push(b'\t'),
        b'u' => {
            let c = match try!(read.decode_hex_escape()) {
                0xDC00...0xDFFF => {
                    return error(read, ErrorCode::LoneLeadingSurrogateInHexEscape);
                }

                // Non-BMP characters are encoded as a sequence of
                // two hex escapes, representing UTF-16 surrogates.
                n1 @ 0xD800...0xDBFF => {
                    if try!(next_or_eof(read)) != b'\\' {
                        return error(read, ErrorCode::UnexpectedEndOfHexEscape);
                    }
                    if try!(next_or_eof(read)) != b'u' {
                        return error(read, ErrorCode::UnexpectedEndOfHexEscape);
                    }

                    let n2 = try!(read.decode_hex_escape());

                    if n2 < 0xDC00 || n2 > 0xDFFF {
                        return error(read, ErrorCode::LoneLeadingSurrogateInHexEscape);
                    }

                    let n = (((n1 - 0xD800) as u32) << 10 | (n2 - 0xDC00) as u32) + 0x1_0000;

                    match char::from_u32(n) {
                        Some(c) => c,
                        None => {
                            return error(read, ErrorCode::InvalidUnicodeCodePoint);
                        }
                    }
                }

                n => match char::from_u32(n as u32) {
                    Some(c) => c,
                    None => {
                        return error(read, ErrorCode::InvalidUnicodeCodePoint);
                    }
                },
            };

            scratch.extend_from_slice(c.encode_utf8(&mut [0_u8; 4]).as_bytes());
        }
        _ => {
            return error(read, ErrorCode::InvalidEscape);
        }
    }

    Ok(())
}

/// Parses a edn escape sequence and discards the value. Assumes the previous
/// byte read was a backslash.
fn ignore_escape<'de, R: ?Sized + Read<'de>>(read: &mut R) -> Result<()> {
    let ch = try!(next_or_eof(read));

    match ch {
        b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => {}
        b'u' => {
            let n = match try!(read.decode_hex_escape()) {
                0xDC00...0xDFFF => {
                    return error(read, ErrorCode::LoneLeadingSurrogateInHexEscape);
                }

                // Non-BMP characters are encoded as a sequence of
                // two hex escapes, representing UTF-16 surrogates.
                n1 @ 0xD800...0xDBFF => {
                    if try!(next_or_eof(read)) != b'\\' {
                        return error(read, ErrorCode::UnexpectedEndOfHexEscape);
                    }
                    if try!(next_or_eof(read)) != b'u' {
                        return error(read, ErrorCode::UnexpectedEndOfHexEscape);
                    }

                    let n2 = try!(read.decode_hex_escape());

                    if n2 < 0xDC00 || n2 > 0xDFFF {
                        return error(read, ErrorCode::LoneLeadingSurrogateInHexEscape);
                    }

                    (((n1 - 0xD800) as u32) << 10 | (n2 - 0xDC00) as u32) + 0x1_0000
                }

                n => n as u32,
            };

            if char::from_u32(n).is_none() {
                return error(read, ErrorCode::InvalidUnicodeCodePoint);
            }
        }
        _ => {
            return error(read, ErrorCode::InvalidEscape);
        }
    }

    Ok(())
}

static HEX: [u8; 256] = {
    const __: u8 = 255; // not a hex digit
    [
        //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 0
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 1
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
        00, 01, 02, 03, 04, 05, 06, 07, 08, 09, __, __, __, __, __, __, // 3
        __, 10, 11, 12, 13, 14, 15, __, __, __, __, __, __, __, __, __, // 4
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 5
        __, 10, 11, 12, 13, 14, 15, __, __, __, __, __, __, __, __, __, // 6
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
    ]
};

fn decode_hex_val(val: u8) -> Option<u16> {
    let n = HEX[val as usize] as u16;
    if n == 255 {
        None
    } else {
        Some(n)
    }
}
