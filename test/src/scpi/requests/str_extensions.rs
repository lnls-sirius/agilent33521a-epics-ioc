use std::cmp;

pub trait StrExtensions {
    fn skip_bytes(&self, bytes_to_skip: usize) -> &Self;
    fn skip_expected_chars(&self, expected: &str) -> &Self;
    fn parse_integer(&self) -> Option<(usize, &Self)>;
}

impl StrExtensions for str {
    fn skip_bytes(&self, bytes_to_skip: usize) -> &str {
        let (_skipped_bytes, remaining_bytes) = self.split_at(bytes_to_skip);

        remaining_bytes
    }

    fn skip_expected_chars(&self, expected: &str) -> &str {
        let paired_chars = self.char_indices().zip(expected.chars());

        let mut indices_of_different_chars = paired_chars.filter_map(
            |((index, a), b)| if a != b { Some(index) } else { None },
        );

        let bytes_to_skip = indices_of_different_chars
            .next()
            .unwrap_or_else(|| cmp::min(self.len(), expected.len()));

        self.skip_bytes(bytes_to_skip)
    }

    fn parse_integer(&self) -> Option<(usize, &str)> {
        let mut non_digit_chars_indices = self.char_indices().filter_map(
            |(index, c)| if c.is_digit(10) { None } else { Some(index) },
        );

        let bytes_to_skip =
            non_digit_chars_indices.next().unwrap_or_else(|| self.len());

        if bytes_to_skip > 0 {
            let (number_string, remaining_bytes) = self.split_at(bytes_to_skip);

            if let Ok(number) = number_string.parse() {
                return Some((number, remaining_bytes));
            }
        }

        None
    }
}
