// use super::context::ContextRef;
// use std::cell::Cell;

// #[derive(Debug, PartialEq, Eq)]
// pub struct LinkedListLink<T> {
//     curr: Cell<Option<ContextRef<T>>>,
//     prev: Cell<Option<ContextRef<T>>>,
//     next: Cell<Option<ContextRef<T>>>,
// }

// impl<T> LinkedListLink<T> {
//     pub fn new() -> Self {
//         LinkedListLink {
//             curr: Cell::new(None),
//             prev: Cell::new(None),
//             next: Cell::new(None),
//         }
//     }

//     fn get_curr(&self) -> Option<ContextRef<T>> {
//         self.curr.get()
//     }

//     fn get_prev(&self) -> Option<ContextRef<T>> {
//         self.prev.get()
//     }

//     fn get_next(&self) -> Option<ContextRef<T>> {
//         self.next.get()
//     }

//     fn set_curr(&self, curr: Option<ContextRef<T>>) {
//         self.curr.set(curr);
//     }

//     fn set_prev(&self, prev: Option<ContextRef<T>>) {
//         self.prev.set(prev);
//     }

//     fn set_next(&self, next: Option<ContextRef<T>>) {
//         self.next.set(next);
//     }
// }

// pub trait LinkedList: Sized {
//     fn get_link(&self) -> &LinkedListLink<Self>;

//     fn prev(&self) -> Option<ContextRef<Self>> {
//         self.get_link().get_prev()
//     }

//     fn next(&self) -> Option<ContextRef<Self>> {
//         self.get_link().get_next()
//     }

//     fn insert_after(&self, next: ContextRef<Self>) {
//         let curr_link = self.get_link();
//         let curr_next = curr_link.get_next();
//         {
//             let next_link = next.get_link();
//             next_link.set_curr(Some(next));
//             next_link.set_prev(curr_link.get_curr());
//             next_link.set_next(curr_next);
//         }
//         if let Some(curr_next) = curr_next {
//             let curr_next_link = curr_next.get_link();
//             curr_next_link.set_prev(Some(next));
//         }
//         curr_link.set_next(Some(next));
//     }

//     fn insert_before(&self, prev: ContextRef<Self>) {
//         {
//             let curr_link = self.get_link();
//             let prev_link = prev.get_link();
//             let curr_prev = curr_link.get_prev();
//             prev_link.set_next(curr_link.get_curr());
//             prev_link.set_prev(curr_prev);
//             if let Some(curr_prev) = curr_prev {
//                 let curr_prev_link = curr_prev.get_link();
//                 curr_prev_link.set_next(Some(prev));
//             }
//             curr_link.set_prev(Some(prev));
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::super::context::Context;
//     use super::*;

//     #[derive(Debug, PartialEq, Eq)]
//     struct Node {
//         value: i32,
//         link: LinkedListLink<Self>,
//     }

//     impl Node {
//         pub fn new(ctx: &Context, value: i32) -> ContextRef<Self> {
//             let node = ctx.create(Node {
//                 value,
//                 link: LinkedListLink::new(),
//             });
//             node.get_link().set_curr(Some(node));
//             node
//         }
//     }

//     impl LinkedList for Node {
//         fn get_link(&self) -> &LinkedListLink<Self> {
//             &self.link
//         }
//     }

//     fn inner(ctx: &Context) {
//         let n1 = Node::new(ctx, 1);
//         let n2 = Node::new(ctx, 2);
//         let n3 = Node::new(ctx, 3);
//         n1.insert_after(n2);
//         assert_eq!(n1.value, 1);
//         assert_eq!(n2.value, 2);
//         assert_eq!(n1.prev(), None);
//         assert_ne!(n1.next(), None);
//         assert_ne!(n2.prev(), None);
//         assert_eq!(n2.next(), None);
//         assert_eq!(n1.next().map(|v| v.value), Some(2));
//         assert_eq!(n2.prev().map(|v| v.value), Some(1));
//         assert_eq!(
//             n1.next().map(|v| v.prev()).flatten().map(|v| v.value),
//             Some(1)
//         );

//         n3.insert_before(n2);

//         assert_ne!(n2.next(), None);
//         assert_ne!(n3.prev(), None);
//         assert_eq!(n3.next(), None);

//         assert_eq!(n2.next().map(|v| v.value), Some(3));
//         assert_eq!(n3.prev().map(|v| v.value), Some(2));
//     }

//     #[test]
//     fn test_basic() {
//         let ctx = Context::new();
//         inner(&ctx);
//     }
// }
