use super::*;
use axgeom::AxisTrait;
use compt::CTreeIterator;
use is_sorted::IsSorted;




///Returns the fraction of bots that are in each level of the tree.
pub fn compute_tree_health<A:AxisTrait,N,T:HasAabb>(tree:&DynTree<A,N,T>)->Vec<f64>{
    
    let mut ratios=vec![0.0;tree.get_height()];

    tree.get_iter().with_depth(compt::Depth(0)).dfs_preorder(|(depth,node),extra|{
        ratios[depth.0]+=node.range.len() as f64;
    });

    let total=tree.get_num_bots() as f64;
    for b in ratios.iter_mut(){
        *b=*b/total;
    }

    return ratios;
}
