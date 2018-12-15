#![feature(trusted_len)]



extern crate axgeom;
extern crate dinotree;
use dinotree::*;
extern crate compt;
use compt::*;


fn assert_length<I:std::iter::TrustedLen>(it:I){
	let len=it.size_hint().0;

	assert_eq!(it.count(),len);
}


#[test]
fn test_zero_sized(){
	let bots=vec![();1];

	let tree=DinoTree::new(axgeom::YAXISS,(),&bots,|_b|{
        axgeom::Rect::new(0isize,0,0,0)
    });

    let n=tree.as_ref().into_vistr().next();
    assert_eq!(n.1.is_none(),true);
    assert_eq!(n.0.range.len(),1);

}

#[test]
fn test_one(){
	let bots=vec![0usize;1];

	let tree=DinoTree::new(axgeom::YAXISS,(),&bots,|_b|{
        axgeom::Rect::new(0isize,0,0,0)
    });

    let n=tree.as_ref().into_vistr().next();
    assert_eq!(n.1.is_none(),true);
    assert_eq!(n.0.range.len(),1);
}


#[test]
fn test_many(){
    let bots=vec![0usize;1000];

    let tree=DinoTree::new(axgeom::YAXISS,(),&bots,|_b|{
        axgeom::Rect::new(0isize,0,0,0)
    });

    assert_eq!(tree.as_ref().vistr().dfs_inorder_iter().count(),tree.as_ref().num_nodes());
    
    let mut num_div=0;
    for (_,b) in tree.as_ref().vistr().dfs_inorder_iter(){
        if let Some(nonleaf)=b{
            if let Some(non_empty)=nonleaf{
                num_div+=1;
                assert_eq!(non_empty.div,0);   
            }
        }

    }
    assert_eq!(num_div,1);
}


#[test]
fn test_empty(){
	let bots:Vec<()>=Vec::new();

	let tree=DinoTree::new(axgeom::YAXISS,(),&bots,|_b|{
        axgeom::Rect::new(0isize,0,0,0)
    });

    let n=tree.as_ref().into_vistr().next();
    assert_eq!(n.0.range.len(),0);
}

#[test]
fn test_iter(){
	let bots=vec![0usize;1234];

    let tree=DinoTree::new(axgeom::YAXISS,(),&bots,|_b|{
        axgeom::Rect::new(0isize,0,0,0)
    });


    let mut last=None;
    for b in tree.as_ref().iter(){
    	match last{
    		None=>{
    			last=Some(b as *const BBox<isize,usize>)
    		},
    		Some(ll)=>{
    			let bb=b as *const BBox<isize,usize>;
    			assert!((ll as usize)<(bb as usize));
    			last=Some(bb)
    		}
    	}
    }	
}

#[test]
fn test_iter2(){
	let bots=vec![0usize;1234];

    let tree=DinoTree::new(axgeom::YAXISS,(),&bots,|_b|{
        axgeom::Rect::new(0isize,0,0,0)
    });

    let num_bots=bots.len();
    assert_eq!(tree.as_ref().iter().count(),num_bots);
	
}
#[test]
fn test(){

	let bots=vec![0usize;1234];

    let mut tree=DinoTree::new(axgeom::YAXISS,(),&bots,|_b|{
        axgeom::Rect::new(0isize,0,0,0)
    });
    dinotree::advanced::are_invariants_met(tree.as_ref()).unwrap();

	assert_length(tree.as_ref_mut().vistr_mut().dfs_preorder_iter());
	assert_length(tree.as_ref().vistr().dfs_preorder_iter());

	let num_nodes=tree.as_ref().num_nodes();

	assert_eq!(tree.as_ref_mut().vistr_mut().dfs_preorder_iter().size_hint().0,num_nodes);

	assert_eq!(tree.as_ref().vistr().bfs_iter().size_hint().0,num_nodes);


	recc(tree.as_ref_mut().vistr_mut());
	//recursively check that the length is correct at each node.
	fn recc(a:VistrMut<(),BBox<isize,usize>>){
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