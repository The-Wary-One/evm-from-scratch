use super::Bytesize;

#[derive(Debug)]
pub struct Calldata<'a>(&'a [u8]);

impl<'a> Calldata<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self(data)
    }

    pub(crate) fn load(&self, offset: usize, size: usize) -> Box<[u8]> {
        let mut bytes = vec![0x00; size];
        for n in 0..size {
            let b = self.0.get(offset + n).unwrap_or(&0);
            bytes[n] = *b;
        }
        bytes.into_boxed_slice()
    }

    pub(crate) fn load_word(&self, offset: usize) -> [u8; 0x20] {
        let mut bytes = [0x00; 0x20];
        for n in 0..=<usize>::from(Bytesize::MAX) {
            let b = self.0.get(offset + n).unwrap_or(&0);
            bytes[n] = *b;
        }
        bytes
    }

    pub(crate) fn size(&self) -> usize {
        self.0.len()
    }
}

impl<'a> From<&Calldata<'a>> for &'a [u8] {
    fn from(c: &Calldata<'a>) -> Self {
        c.0
    }
}

impl<'a> From<&Calldata<'a>> for Box<[u8]> {
    fn from(c: &Calldata<'a>) -> Self {
        c.load(0, c.size())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_correctly_load_word() {
        let bytes =
            hex::decode("000000FF0000000000000000000000000000000000000000000000000000000000001234")
                .unwrap();
        let cd = Calldata::new(&bytes);
        assert_eq!(
            &cd.load_word(0)[..],
            &hex::decode("000000FF00000000000000000000000000000000000000000000000000000000")
                .unwrap()[..]
        );
        assert_eq!(
            &cd.load_word(4)[..],
            &hex::decode("0000000000000000000000000000000000000000000000000000000000001234")
                .unwrap()[..]
        );
        assert_eq!(
            &cd.load_word(34)[..],
            &hex::decode("1234000000000000000000000000000000000000000000000000000000000000")
                .unwrap()[..]
        );
    }
}
