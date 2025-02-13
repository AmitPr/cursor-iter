use std::str::CharIndices;

fn is_newline(c: char) -> bool {
    c == '\n'
}

/// A cursor that can move both forward and backward through a string.
#[derive(Debug, Clone)]
pub struct Cursor<'a> {
    data: &'a str,
    offset: usize,
    line: usize,
    forward: CharIndices<'a>,
}

impl<'a> Cursor<'a> {
    pub fn new(data: &'a str) -> Self {
        Self {
            data,
            offset: 0,
            line: 0,
            forward: data.char_indices(),
        }
    }

    fn backward(&self) -> CharIndices<'a> {
        self.data[..self.offset].char_indices()
    }

    pub fn next_char(&mut self) -> Option<char> {
        self.next().map(|(_, c)| c)
    }

    pub fn next_word(&mut self) -> Option<(usize, &'a str)> {
        let start = self.offset;
        while let Some((_, c)) = self.peek() {
            if c.is_whitespace() {
                break;
            }
            self.next();
        }
        let end = self.offset;
        if start < end {
            Some((start, &self.data[start..=end]))
        } else {
            None
        }
    }

    pub fn next_line(&mut self) -> Option<&'a str> {
        let start = self.offset;
        while let Some((_, c)) = self.peek() {
            if is_newline(c) {
                break;
            }
            self.next();
        }
        let end = self.offset;
        if start < end {
            Some(&self.data[start..=end])
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some((_, c)) = self.peek() {
            if !c.is_whitespace() {
                break;
            }
            self.next();
        }
    }

    pub fn prev(&mut self) -> Option<(usize, char)> {
        let mut backward = self.backward();

        let last_byte_len = backward.as_str().as_bytes().len();
        let (pos, c) = backward.next_back()?;
        let cur_byte_len = backward.as_str().as_bytes().len();
        self.offset -= last_byte_len - cur_byte_len;

        self.forward = self.data[self.offset..].char_indices();

        if is_newline(c) {
            self.line -= 1;
        }

        Some((pos, c))
    }

    pub fn prev_char(&mut self) -> Option<char> {
        self.prev().map(|(_, c)| c)
    }

    pub fn peek(&self) -> Option<(usize, char)> {
        self.forward.clone().next()
    }

    pub fn peek_char(&self) -> Option<char> {
        self.peek().map(|(_, c)| c)
    }

    pub fn lookback(&self) -> Option<(usize, char)> {
        self.backward().next_back()
    }

    pub fn lookback_char(&self) -> Option<char> {
        self.lookback().map(|(_, c)| c)
    }

    pub const fn line(&self) -> usize {
        self.line
    }

    pub const fn words(&mut self) -> CursorWords<'a, '_> {
        CursorWords::new(self)
    }

    pub const fn words_with_lines(&mut self) -> CursorWords<'a, '_, true> {
        CursorWords::with_lines(self)
    }

    pub const fn lines(&mut self) -> CursorLines<'a, '_> {
        CursorLines::new(self)
    }
}

impl Iterator for Cursor<'_> {
    type Item = (usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        let last_byte_len = self.forward.as_str().as_bytes().len();
        let (pos, c) = self.forward.next()?;
        let cur_byte_len = self.forward.as_str().as_bytes().len();
        self.offset += last_byte_len - cur_byte_len;

        if is_newline(c) {
            self.line += 1;
        }

        Some((pos, c))
    }
}

pub struct CursorWords<'a, 'b, const LINES: bool = false> {
    cursor: &'b mut Cursor<'a>,
}

impl<'a, 'b> CursorWords<'a, 'b> {
    pub const fn new(cursor: &'b mut Cursor<'a>) -> Self {
        Self { cursor }
    }
}

impl<'a, 'b> CursorWords<'a, 'b, true> {
    pub const fn with_lines(cursor: &'b mut Cursor<'a>) -> Self {
        Self { cursor }
    }
}

impl<'a> Iterator for CursorWords<'a, '_, false> {
    type Item = (usize, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.cursor.next_word()?;
        self.cursor.skip_whitespace();
        Some(ret)
    }
}

impl<'a> Iterator for CursorWords<'a, '_, true> {
    type Item = (usize, usize, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.cursor.line();
        let (offset, word) = self.cursor.next_word()?;
        self.cursor.skip_whitespace();
        Some((offset, line, word))
    }
}

pub struct CursorLines<'a, 'b> {
    cursor: &'b mut Cursor<'a>,
}

impl<'a, 'b> CursorLines<'a, 'b> {
    pub const fn new(cursor: &'b mut Cursor<'a>) -> Self {
        Self { cursor }
    }
}

impl<'a> Iterator for CursorLines<'a, '_> {
    type Item = (usize, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.cursor.line();
        let ret = self.cursor.next_line()?;
        Some((line, ret))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cursor() {
        let cursor = Cursor::new("hello");
        assert_eq!(cursor.peek_char(), Some('h'));
        assert_eq!(cursor.lookback_char(), None);
    }

    #[test]
    fn test_empty_string() {
        let mut cursor = Cursor::new("");
        assert_eq!(cursor.next_char(), None);
        assert_eq!(cursor.prev_char(), None);
        assert_eq!(cursor.peek_char(), None);
        assert_eq!(cursor.lookback_char(), None);
    }

    #[test]
    fn test_advance() {
        let mut cursor = Cursor::new("abc");

        assert_eq!(cursor.next(), Some((0, 'a')));
        assert_eq!(cursor.next(), Some((1, 'b')));
        assert_eq!(cursor.next(), Some((2, 'c')));
        assert_eq!(cursor.next(), None);
    }

    #[test]
    fn test_prev() {
        let mut cursor = Cursor::new("abc");

        // Advance to end
        cursor.next();
        cursor.next();
        cursor.next();

        assert_eq!(cursor.prev(), Some((2, 'c')));
        assert_eq!(cursor.prev(), Some((1, 'b')));
        assert_eq!(cursor.prev(), Some((0, 'a')));
        assert_eq!(cursor.prev(), None);
    }

    #[test]
    fn test_peek() {
        let mut cursor = Cursor::new("abc");

        assert_eq!(cursor.peek(), Some((0, 'a')));
        cursor.next();
        assert_eq!(cursor.peek(), Some((1, 'b')));
        cursor.next();
        assert_eq!(cursor.peek(), Some((2, 'c')));
        cursor.next();
        assert_eq!(cursor.peek(), None);
    }

    #[test]
    fn test_lookback() {
        let mut cursor = Cursor::new("abc");

        assert_eq!(cursor.lookback(), None);
        cursor.next();
        assert_eq!(cursor.lookback(), Some((0, 'a')));
        cursor.next();
        assert_eq!(cursor.lookback(), Some((1, 'b')));
    }

    #[test]
    fn test_bidirectional_movement() {
        let mut cursor = Cursor::new("hello");

        assert_eq!(cursor.next_char(), Some('h'));
        assert_eq!(cursor.next_char(), Some('e'));
        assert_eq!(cursor.prev_char(), Some('e'));
        assert_eq!(cursor.prev_char(), Some('h'));
        assert_eq!(cursor.next_char(), Some('h'));
    }

    #[test]
    fn test_unicode() {
        let mut cursor = Cursor::new("hello 👋 world");

        // Advance to emoji
        for _ in 0..6 {
            cursor.next();
        }

        assert_eq!(cursor.next_char(), Some('👋'));
        assert_eq!(cursor.prev_char(), Some('👋'));
    }

    #[test]
    fn test_iterator_implementation() {
        let cursor = Cursor::new("abc");
        let collected: Vec<(usize, char)> = cursor.collect();

        assert_eq!(collected, vec![(0, 'a'), (1, 'b'), (2, 'c'),]);
    }

    #[test]
    fn test_mixed_operations() {
        let mut cursor = Cursor::new("test");

        assert_eq!(cursor.next_char(), Some('t'));
        assert_eq!(cursor.peek_char(), Some('e'));
        assert_eq!(cursor.prev_char(), Some('t'));
        assert_eq!(cursor.lookback_char(), None);
        assert_eq!(cursor.next_char(), Some('t'));
        assert_eq!(cursor.next_char(), Some('e'));
        assert_eq!(cursor.lookback_char(), Some('e'));
    }
}
