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


	recc(tree.get_iter_mut());
	//recursively check that the length is correct at each node.
	fn recc(a:NdIterMut<(),BBox<isize,usize>>){
		let (_nn,rest)=a.next();
		match rest{
			Some((_extra,mut left,mut right))=>{
				{
					let left=left.create_wrap_mut();
					let right=right.create_wrap_mut();
					assert_length(left.dfs_preorder_iter());
					assert_length(right.dfs_preorder_iter());
				}
				recc(left);
				recc(right);
			},
			None=>{

			}
		}
	}
}