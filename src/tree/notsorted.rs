use crate::inner_prelude::*;

///A version of a dinotree where the bots that belong to a node are not
///sorted along an axis. So this is really a regular kd-tree.
pub struct NotSorted<A: AxisTrait, T: HasAabb>(pub DinoTree<A, T>);
