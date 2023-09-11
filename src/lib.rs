
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use syn::{parse::Parse, Attribute, Item, ItemMod, ItemStruct, ItemTrait};

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
struct MarkedItem<T> {
    pub mark: Attribute,
    pub item: T,
}

type SharedMarkedItem<T> = MarkedItem<Rc<RefCell<T>>>;

impl<T> MarkedItem<T> {
    pub fn new(mark: Attribute, item: T) -> Self {
        Self { mark, item }
    }
}

/// Returns the index of the first [Attribute] that contains a given name if found
fn find_attribute(attrs: &[Attribute], name: &str) -> Option<(usize, String)> {
    for (index, struct_attrib) in attrs.iter().enumerate() {
        let path = struct_attrib.path();

        match path.get_ident() {
            Some(ident) => {
                let ident = ident.to_string();
                if ident.contains(name) {
                    return Some((index, ident));
                }
            }
            None => (),
        }
    }

    None
}

/// Return all Items that contain the given mark, also removes the mark from the item and return it
/// as a [MarkedItem]
///
/// We use a attribute macro as a way to mark items, so that we can further process them in the
/// proc_macros
fn get_items_by_mark_prefix<'a>(
    items: &[Rc<RefCell<Item>>],
    mark: &'a str,
) -> HashMap<String, Vec<SharedMarkedItem<Item>>> {
    let mut marked_items: HashMap<String, Vec<SharedMarkedItem<Item>>> = HashMap::new();

    use Item as I;

    for item in items {
        let mut i = item.borrow_mut();
        let attrs = match *i {
            I::Struct(ItemStruct { ref mut attrs, .. }) => attrs,
            I::Trait(ItemTrait { ref mut attrs, .. }) => attrs,
            I::Mod(syn::ItemMod { ref mut attrs, .. }) => attrs,
            I::Enum(syn::ItemEnum { ref mut attrs, .. }) => attrs,
            I::Fn(syn::ItemFn { ref mut attrs, .. }) => attrs,
            _ => continue,
        };

        if let Some((indx, attr_ident)) = find_attribute(attrs, mark) {
            let a = attrs.remove(indx);
            let marked_item = MarkedItem::new(a, item.clone());
            match marked_items.get_mut(&attr_ident) {
                Some(marked) => marked.push(marked_item),
                None => {
                    marked_items.insert(attr_ident, vec![marked_item]);
                }
            };
        }
    }

    marked_items
}

#[derive(Debug, Clone, Default)]
struct MacroScope {
    pub items: Vec<Rc<RefCell<Item>>>,
}

impl Parse for MacroScope {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let item: ItemMod = input.parse()?;

        let items = match item.content {
            Some(c) => c.1,
            None => return Ok(Default::default()),
        };

        let items: Vec<_> = items
            .into_iter()
            .map(|item| Rc::new(RefCell::new(item)))
            .collect();

        Ok(Self { items })
    }
}
