 #![feature(test)]
#![feature(trusted_len)]
extern crate axgeom;
extern crate dinotree_inner;
use dinotree_inner::*;
extern crate compt;
use compt::*;

fn assert_length<I:std::iter::TrustedLen>(it:I){
	let len=it.size_hint().0;

	assert_eq!(it.count(),len);
}

#[test]
fn test(){

	let bots=vec![0usize;1234];

    let mut tree=DynTree::new(axgeom::YAXISS,(),&bots,|_b|{
        axgeom::Rect::new(0isize,0,0,0)
    });
	tree.are_invariants_met().unwrap();

	assert_length(tree.get_iter_mut().dfs_preorder_iter());
	assert_length(tree.get_iter().dfs_preorder_iter());

	let num_nodes=tree.get_num_nodes();

	assert_eq!(tree.get_iter_mut().dfs_preorder_iter().size_hint().0,num_nodes);

	assert_eq!(tree.get_iter().bfs_iter().size_hint().0,num_nodes);
}