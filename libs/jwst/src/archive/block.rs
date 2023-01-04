#![allow(deprecated)]
use super::workspace::Content;
use crate::{utils::JS_INT_RANGE, BlockHistory, HistoryOperation};

use super::*;
use lib0::any::Any;

use std::collections::HashMap;
use yrs::{
    types::ToJson, Array, ArrayPrelim, ArrayRef, Map, MapPrelim, MapRef, ReadTxn, Transaction,
    TransactionMut,
};

#[derive(Debug, PartialEq)]
#[deprecated = "Use OctoBlockRef"]
pub struct Block {
    // block schema
    // for example: {
    //     title: {type: 'string', default: ''}
    //     description: {type: 'string', default: ''}
    // }
    // default_props: HashMap<String, BlockField>,
    id: String,
    // Question: What is the point of an operator, here?
    //  Shouldn't the operator be supplied via the corresponding Workspace? / Transaction?
    operator: u64,
    block: MapRef,
    children: ArrayRef,
    // This updated is informed by the Workspace updated MapRef on construction
    updated: ArrayRef,
}

unsafe impl Send for Block {}

impl Block {
    // Create a new block, skip create if block is already created.
    pub fn new<B, F>(
        workspace: &Workspace,
        trx: &TransactionMut,
        block_id: B,
        flavor: F,
        operator: u64,
    ) -> Block
    where
        B: AsRef<str>,
        F: AsRef<str>,
    {
        let block_id = block_id.as_ref();
        workspace.with_trx(|mut trx| {
            if let Some(block) = Self::from(workspace.content(), &trx.trx_mut, block_id, operator) {
                block
            } else {
                // init base struct
                workspace
                    .blocks()
                    .insert(&mut trx.trx_mut, block_id, MapPrelim::<Any>::new());
                let block = workspace
                    .blocks()
                    .get(&mut trx.trx_mut, block_id)
                    .and_then(|b| b.to_ymap())
                    .unwrap();

                // init default schema
                block.insert(&mut trx.trx_mut, "sys:flavor", flavor.as_ref());
                block.insert(&mut trx.trx_mut, "sys:version", ArrayPrelim::from([1, 0]));
                block.insert(
                    &mut trx.trx_mut,
                    "sys:children",
                    ArrayPrelim::<Vec<String>, String>::from(vec![]),
                );
                block.insert(
                    &mut trx.trx_mut,
                    "sys:created",
                    chrono::Utc::now().timestamp_millis() as f64,
                );

                workspace.updated().insert(
                    &mut trx.trx_mut,
                    block_id,
                    ArrayPrelim::<_, Any>::from([]),
                );

                let children = block
                    .get(&trx.trx_mut, "sys:children")
                    .and_then(|c| c.to_yarray())
                    .unwrap();
                let updated = workspace
                    .updated()
                    .get(&trx.trx_mut, block_id)
                    .and_then(|c| c.to_yarray())
                    .unwrap();

                let block = Self {
                    id: block_id.to_string(),
                    operator,
                    block,
                    children,
                    updated,
                };

                block.log_update(&mut trx.trx_mut, HistoryOperation::Add);

                block
            }
        })
    }

    pub fn from<B, T>(workspace: &Content, trx: &T, block_id: B, operator: u64) -> Option<Block>
    where
        B: AsRef<str>,
        T: ReadTxn,
    {
        let block = workspace.blocks().get(trx, block_id.as_ref())?.to_ymap()?;
        let updated = workspace
            .updated()
            .get(trx, block_id.as_ref())?
            .to_yarray()?;

        let children = block.get(trx, "sys:children")?.to_yarray()?;
        Some(Self {
            id: block_id.as_ref().to_string(),
            operator,
            block,
            children,
            updated,
        })
    }

    pub fn from_raw_parts(
        id: String,
        txn: &Transaction,
        block: MapRef,
        updated: ArrayRef,
        operator: u64,
    ) -> Block {
        let children = block.get(txn, "sys:children").unwrap().to_yarray().unwrap();
        Self {
            id,
            operator,
            block,
            children,
            updated,
        }
    }

    pub(crate) fn log_update(&self, trx: &mut TransactionMut, action: HistoryOperation) {
        let array = ArrayPrelim::from([
            Any::Number(self.operator as f64),
            Any::Number(chrono::Utc::now().timestamp_millis() as f64),
            Any::String(Box::from(action.to_string())),
        ]);

        self.updated.push_back(trx, array);
    }

    pub fn get<T: ReadTxn>(&self, trx: &T, key: &str) -> Option<Any> {
        let key = format!("prop:{}", key);
        self.block
            .get(trx, &key)
            .and_then(|v| match v.to_json(trx) {
                Any::Null | Any::Undefined | Any::Array(_) | Any::Buffer(_) | Any::Map(_) => {
                    log::error!("get wrong value at key {}", key);
                    None
                }
                v => Some(v),
            })
    }

    pub fn set<T>(&self, trx: &mut TransactionMut, key: &str, value: T)
    where
        T: Into<Any>,
    {
        let key = format!("prop:{}", key);
        match value.into() {
            Any::Bool(bool) => {
                self.block.insert(trx, key, bool);
                self.log_update(trx, HistoryOperation::Update);
            }
            Any::String(text) => {
                self.block.insert(trx, key, text.to_string());
                self.log_update(trx, HistoryOperation::Update);
            }
            Any::Number(number) => {
                self.block.insert(trx, key, number);
                self.log_update(trx, HistoryOperation::Update);
            }
            Any::BigInt(number) => {
                if JS_INT_RANGE.contains(&number) {
                    self.block.insert(trx, key, number as f64);
                } else {
                    self.block.insert(trx, key, number);
                }
                self.log_update(trx, HistoryOperation::Update);
            }
            Any::Null | Any::Undefined => {
                self.block.remove(trx, &key);
                self.log_update(trx, HistoryOperation::Delete);
            }
            Any::Buffer(_) | Any::Array(_) | Any::Map(_) => {}
        }
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    // start with a namespace
    // for example: affine:text
    pub fn flavor<T>(&self, trx: &T) -> String
    where
        T: ReadTxn,
    {
        self.block
            .get(trx, "sys:flavor")
            .unwrap_or_default()
            .to_string(trx)
    }

    // block schema version
    // for example: [1, 0]
    pub fn version<T>(&self, trx: &T) -> [usize; 2]
    where
        T: ReadTxn,
    {
        self.block
            .get(trx, "sys:version")
            .and_then(|v| v.to_yarray())
            .map(|v| {
                v.iter(trx)
                    .take(2)
                    .filter_map(|s| s.to_string(trx).parse::<usize>().ok())
                    .collect::<Vec<_>>()
            })
            .unwrap()
            .try_into()
            .unwrap()
    }

    pub fn created<T>(&self, trx: &T) -> u64
    where
        T: ReadTxn,
    {
        self.block
            .get(trx, "sys:created")
            .and_then(|c| match c.to_json(trx) {
                Any::Number(n) => Some(n as u64),
                _ => None,
            })
            .unwrap_or_default()
    }

    pub fn updated<T>(&self, trx: &T) -> u64
    where
        T: ReadTxn,
    {
        self.updated
            .iter(trx)
            .filter_map(|v| v.to_yarray())
            .last()
            .and_then(|a| {
                a.get(trx, 1).and_then(|i| match i.to_json(trx) {
                    Any::Number(n) => Some(n as u64),
                    _ => None,
                })
            })
            .unwrap_or_else(|| self.created(trx))
    }

    pub fn history<T>(&self, trx: &T) -> Vec<BlockHistory>
    where
        T: ReadTxn,
    {
        self.updated
            .iter(trx)
            .filter_map(|v| v.to_yarray())
            .map(
                |v| todo!(), // (v, trx, self.id.clone()).into()
            )
            .collect()
    }

    pub fn parent<T: ReadTxn>(&self, trx: &T) -> Option<String> {
        self.block
            .get(trx, "sys:parent")
            .and_then(|c| match c.to_json(trx) {
                Any::String(s) => Some(s.to_string()),
                _ => None,
            })
    }

    pub fn children<T: ReadTxn>(&self, trx: &T) -> Vec<String> {
        self.children.iter(trx).map(|v| v.to_string(trx)).collect()
    }

    #[inline]
    pub fn children_iter<'a, T: ReadTxn>(
        &'a self,
        trx: &'a T,
    ) -> impl Iterator<Item = String> + 'a {
        self.children.iter(trx).map(|v| v.to_string(trx))
    }

    pub fn children_len<T: ReadTxn>(&self, trx: &T) -> u32 {
        self.children.len(trx)
    }

    pub(crate) fn content<T: ReadTxn>(&self, trx: &T) -> HashMap<String, Any> {
        self.block
            .iter(trx)
            .filter_map(|(key, val)| {
                if key.starts_with("prop:") {
                    Some((key[5..].to_owned(), val.to_json(trx)))
                } else {
                    None
                }
            })
            .collect()
    }

    fn set_parent(&self, trx: &mut TransactionMut, block_id: String) {
        self.block.insert(trx, "sys:parent", block_id);
    }

    pub fn push_children(&self, trx: &mut TransactionMut, block: &Block) {
        self.remove_children(trx, block);
        block.set_parent(trx, self.id.clone());

        self.children.push_back(trx, block.id.clone());

        self.log_update(trx, HistoryOperation::Add);
    }

    pub fn insert_children_at(&self, trx: &mut TransactionMut, block: &Block, pos: u32) {
        self.remove_children(trx, block);
        block.set_parent(trx, self.id.clone());

        let children = &self.children;

        if children.len(trx) > pos {
            children.insert(trx, pos, block.id.clone());
        } else {
            children.push_back(trx, block.id.clone());
        }

        self.log_update(trx, HistoryOperation::Add);
    }

    pub fn insert_children_before(&self, trx: &mut TransactionMut, block: &Block, reference: &str) {
        self.remove_children(trx, block);
        block.set_parent(trx, self.id.clone());

        let children = &self.children;

        if let Some(pos) = children
            .iter(trx)
            .position(|c| c.to_string(trx) == reference)
        {
            children.insert(trx, pos as u32, block.id.clone());
        } else {
            children.push_back(trx, block.id.clone());
        }

        self.log_update(trx, HistoryOperation::Add);
    }

    pub fn insert_children_after(&self, trx: &mut TransactionMut, block: &Block, reference: &str) {
        self.remove_children(trx, block);
        block.set_parent(trx, self.id.clone());

        let children = &self.children;

        match children
            .iter(trx)
            .position(|c| c.to_string(trx) == reference)
        {
            Some(pos) if (pos as u32) < children.len(trx) => {
                children.insert(trx, pos as u32 + 1, block.id.clone());
            }
            _ => {
                children.push_back(trx, block.id.clone());
            }
        }

        self.log_update(trx, HistoryOperation::Add);
    }

    pub fn remove_children(&self, trx: &mut TransactionMut, block: &Block) {
        let children = &self.children;
        block.set_parent(trx, self.id.clone());

        if let Some(current_pos) = children
            .iter(trx)
            .position(|c| c.to_string(trx) == block.id)
        {
            children.remove(trx, current_pos as u32);
            self.log_update(trx, HistoryOperation::Delete);
        }
    }

    pub fn exists_children<T: ReadTxn>(&self, trx: &T, block_id: &str) -> Option<usize> {
        self.children
            .iter(trx)
            .position(|c| c.to_string(trx) == block_id)
    }
}

// Cannot serialize a block out of the document
// impl Serialize for Block {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         // Would need a document/transaction to get to this
//         let any = self.block.to_json();
//         any.serialize(serializer)
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn init_block() {
        let workspace = Workspace::new("test");

        let mut trx = workspace.get_trx();

        // new block
        let block = trx.create("test", "affine:text");
        assert_eq!(block.id(), "test");
        assert_eq!(block.flavor(&trx.trx_mut), "affine:text");
        assert_eq!(block.version(&trx.trx_mut), [1, 0]);

        // get exist block
        let block = trx.get("test").unwrap();
        assert_eq!(block.flavor(&trx.trx_mut), "affine:text");
        assert_eq!(block.version(&trx.trx_mut), [1, 0]);
    }

    #[test]
    fn set_value() {
        let workspace = Workspace::new("test");

        workspace.with_trx(|mut t| {
            let block = t.create("test", "affine:text");

            let trx = &mut t.trx_mut;

            // normal type set
            block.set(trx, "bool", true);
            block.set(trx, "text", "hello world");
            block.set(trx, "text_owned", "hello world".to_owned());
            block.set(trx, "num", 123_i64);
            block.set(trx, "bigint", 9007199254740992_i64);

            assert_eq!(block.get(&t.trx_mut, "bool").unwrap().to_string(), "true");
            assert_eq!(
                block.get(&t.trx_mut, "text").unwrap().to_string(),
                "hello world"
            );
            assert_eq!(
                block.get(&t.trx_mut, "text_owned").unwrap().to_string(),
                "hello world"
            );
            assert_eq!(block.get(&t.trx_mut, "num").unwrap().to_string(), "123");
            assert_eq!(
                block.get(&t.trx_mut, "bigint").unwrap().to_string(),
                "9007199254740992"
            );

            assert_eq!(
                block.content(&t.trx_mut),
                vec![
                    ("bool".to_owned(), Any::Bool(true)),
                    ("text".to_owned(), Any::String("hello world".into())),
                    ("text_owned".to_owned(), Any::String("hello world".into())),
                    ("num".to_owned(), Any::Number(123.0)),
                    ("bigint".to_owned(), Any::BigInt(9007199254740992)),
                ]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>()
            );
        });
    }

    #[test]
    fn insert_remove_children() {
        let workspace = Workspace::new("text");

        workspace.with_trx(|mut t| {
            let block = t.create("a", "affine:text");
            let b = t.create("b", "affine:text");
            let c = t.create("c", "affine:text");
            let d = t.create("d", "affine:text");
            let e = t.create("e", "affine:text");
            let f = t.create("f", "affine:text");
            let trx = &mut t.trx_mut;

            block.push_children(trx, &b);
            block.insert_children_at(trx, &c, 0);
            block.insert_children_before(trx, &d, "b");
            block.insert_children_after(trx, &e, "b");
            block.insert_children_after(trx, &f, "c");

            assert_eq!(
                block.children(trx),
                vec![
                    "c".to_owned(),
                    "f".to_owned(),
                    "d".to_owned(),
                    "b".to_owned(),
                    "e".to_owned()
                ]
            );

            block.remove_children(trx, &d);

            assert_eq!(
                block
                    .children
                    .iter(trx)
                    .map(|i| i.to_string(trx))
                    .collect::<Vec<_>>(),
                vec![
                    "c".to_owned(),
                    "f".to_owned(),
                    "b".to_owned(),
                    "e".to_owned()
                ]
            );
        });
    }

    #[test]
    fn updated() {
        let workspace = Workspace::new("test");
        workspace.with_trx(|mut t| {
            let block = t.create("a", "affine:text");
            block.set(&mut t.trx_mut, "test", 1);

            assert!(block.created(&t.trx_mut) <= block.updated(&t.trx_mut))
        });
    }

    #[test]
    fn history() {
        use yrs::Doc;

        let doc = Doc::with_client_id(123);

        let workspace = Workspace::from_doc(doc, "test");

        let history = workspace.with_trx(|mut t| {
            let block = t.create("a", "affine:text");
            let b = t.create("b", "affine:text");
            let trx = &t.trx_mut;
            block.set(&mut t.trx_mut, "test", 1);

            let history = block.history(&t.trx_mut);

            assert_eq!(history.len(), 2);

            // let history = history.last().unwrap();

            assert_eq!(
                history,
                vec![
                    BlockHistory {
                        block_id: "a".to_owned(),
                        client: 123,
                        timestamp: history.get(0).unwrap().timestamp,
                        operation: HistoryOperation::Add,
                    },
                    BlockHistory {
                        block_id: "a".to_owned(),
                        client: 123,
                        timestamp: history.get(1).unwrap().timestamp,
                        operation: HistoryOperation::Update,
                    }
                ]
            );

            block.push_children(&mut t.trx_mut, &b);

            assert_eq!(block.exists_children(&t.trx_mut, "b"), Some(0));

            block.remove_children(&mut t.trx_mut, &b);

            assert_eq!(block.exists_children(&t.trx_mut, "b"), None);

            let history = block.history(&t.trx_mut);
            assert_eq!(history.len(), 4);

            history
        });

        if let [.., insert, remove] = history.as_slice() {
            assert_eq!(
                insert,
                &BlockHistory {
                    block_id: "a".to_owned(),
                    client: 123,
                    timestamp: insert.timestamp,
                    operation: HistoryOperation::Add,
                }
            );
            assert_eq!(
                remove,
                &BlockHistory {
                    block_id: "a".to_owned(),
                    client: 123,
                    timestamp: remove.timestamp,
                    operation: HistoryOperation::Delete,
                }
            );
        } else {
            assert!(false)
        }
    }
}