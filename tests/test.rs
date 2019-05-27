
extern crate axgeom;
extern crate dinotree;
use dinotree::*;
use dinotree::copy::*;
extern crate compt;
use compt::*;

fn assert_length<I: std::iter::ExactSizeIterator>(it: I) {
    let len = it.size_hint().0;

    assert_eq!(it.count(), len);
}

#[test]
fn test_zero_sized() {
    let bots = vec![(); 1];

    let tree = DinoTreeBuilder::new(axgeom::YAXISS, &bots, |_b| {
        axgeom::Rect::new(0isize, 0, 0, 0)
    })
    .build_seq();

    let (n, _) = tree.vistr().next();
    assert_eq!(n.div.is_none(), true);
    assert_eq!(n.bots.len(), 1);
    assert!(n.cont.is_some());
}

#[test]
fn test_one() {
    let bots = vec![0usize; 1];

    let tree = DinoTreeBuilder::new(axgeom::YAXISS, &bots, |_b| {
        axgeom::Rect::new(0isize, 0, 0, 0)
    })
    .build_seq();

    let (n, _) = tree.vistr().next();
    assert_eq!(n.div.is_none(), true);
    assert_eq!(n.bots.len(), 1);
    assert!(n.cont.is_some())
}

#[test]
fn test_many() {
    let bots = vec![0usize; 1000];

    let tree = DinoTreeBuilder::new(axgeom::YAXISS, &bots, |_b| {
        axgeom::Rect::new(0isize, 0, 0, 0)
    })
    .build_seq();

    assert_eq!(
        tree.vistr().dfs_inorder_iter().count(),
        tree.num_nodes()
    );

    let mut num_div = 0;
    for b in tree.vistr().dfs_inorder_iter() {
        if let Some(_) = b.div {
            if let Some(_) = b.cont {
                num_div += 1;
            }
        }
    }
    assert_eq!(num_div, 1);
}

#[test]
fn test_empty() {
    let bots: Vec<()> = Vec::new();

    let tree = DinoTreeBuilder::new(axgeom::YAXISS, &bots, |_b| {
        axgeom::Rect::new(0isize, 0, 0, 0)
    })
    .build_seq();

    let (n, _) = tree.vistr().next();
    assert_eq!(n.bots.len(), 0);
}

#[test]
fn test_iter() {
    let bots = vec![0usize; 1234];

    let tree = DinoTreeBuilder::new(axgeom::YAXISS, &bots, |_b| {
        axgeom::Rect::new(0isize, 0, 0, 0)
    })
    .build_seq();

    let mut last = None;
    for b in tree.get_bots().iter() {
        match last {
            None => last = Some(b as *const BBox<isize, usize>),
            Some(ll) => {
                let bb = b as *const BBox<isize, usize>;
                assert!((ll as usize) < (bb as usize));
                last = Some(bb)
            }
        }
    }
}

#[test]
fn test_iter2() {
    let bots = vec![0usize; 1234];

    let tree = DinoTreeBuilder::new(axgeom::YAXISS, &bots, |_b| {
        axgeom::Rect::new(0isize, 0, 0, 0)
    })
    .build_seq();

    let num_bots = bots.len();
    assert_eq!(tree.get_bots().iter().count(), num_bots);
}
#[test]
fn test() {
    let bots = vec![0usize; 1234];

    let mut tree = DinoTreeBuilder::new(axgeom::YAXISS, &bots, |_b| {
        axgeom::Rect::new(0isize, 0, 0, 0)
    })
    .build_seq();

    assert!(assert_invariants(&tree));

    assert_length(tree.vistr_mut().dfs_preorder_iter());
    assert_length(tree.vistr().dfs_preorder_iter());

    let num_nodes = tree.num_nodes();

    assert_eq!(
        tree
            .vistr_mut()
            .dfs_preorder_iter()
            .size_hint()
            .0,
        num_nodes
    );

    assert_eq!(tree.vistr().bfs_iter().size_hint().0, num_nodes);

    recc(tree.vistr_mut());
    //recursively check that the length is correct at each node.
    fn recc(a: VistrMut<BBox<isize, usize>>) {
        let (_nn, rest) = a.next();
        match rest {
            Some([mut left, mut right]) => {
                {
                    let left = left.create_wrap_mut();
                    let right = right.create_wrap_mut();
                    assert_length(left.dfs_preorder_iter());
                    assert_length(right.dfs_preorder_iter());
                }
                recc(left);
                recc(right);
            }
            None => {}
        }
    }
}
