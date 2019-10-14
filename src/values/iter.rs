use super::*;

impl<'a> IntoIterator for &'a DictionaryValue {
    type Item = (String, StoredValue);
    type IntoIter = DictionaryValueIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        DictionaryValueIter {
            keys: self.keys().keys,
            dictionary: self,
        }
    }
}

pub struct DictionaryValueIter<'a> {
    pub(crate) keys: CefStringListIntoIter,
    pub(crate) dictionary: &'a DictionaryValue,
}

pub struct DictionaryValueKeysIter<'a> {
    pub(crate) keys: CefStringListIntoIter,
    pub(crate) _dictionary: PhantomData<&'a DictionaryValue>,
}

impl<'a> Iterator for DictionaryValueKeysIter<'a> {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        self.keys.next().map(String::from)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.keys.size_hint()
    }
}
impl ExactSizeIterator for DictionaryValueKeysIter<'_> {}

impl<'a> Iterator for DictionaryValueIter<'a> {
    type Item = (String, StoredValue);
    fn next(&mut self) -> Option<(String, StoredValue)> {
        self.keys.next()
            .map(|key| {
                let value = self.dictionary.get_value_inner(&key).into();
                (key.into(), value)
            })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.keys.size_hint()
    }
}
impl ExactSizeIterator for DictionaryValueIter<'_> {}

impl IntoIterator for ListValue {
    type Item = StoredValue;
    type IntoIter = ListValueIntoIter;
    fn into_iter(self) -> ListValueIntoIter {
        ListValueIntoIter {
            range: 0..self.len(),
            list: self
        }
    }
}

impl<'a> IntoIterator for &'a ListValue {
    type Item = StoredValue;
    type IntoIter = ListValueIter<'a>;
    fn into_iter(self) -> ListValueIter<'a> {
        ListValueIter {
            range: 0..self.len(),
            list: self
        }
    }
}

pub struct ListValueIntoIter {
    range: std::ops::Range<usize>,
    list: ListValue,
}

pub struct ListValueIter<'a> {
    range: std::ops::Range<usize>,
    list: &'a ListValue,
}

impl Iterator for ListValueIntoIter {
    type Item = StoredValue;
    fn next(&mut self) -> Option<StoredValue> {
        self.range.next().and_then(|i| self.list.get(i))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}
impl ExactSizeIterator for ListValueIntoIter {}

impl Iterator for ListValueIter<'_> {
    type Item = StoredValue;
    fn next(&mut self) -> Option<StoredValue> {
        self.range.next().and_then(|i| self.list.get(i))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}
impl ExactSizeIterator for ListValueIter<'_> {}
