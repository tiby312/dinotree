use super::*;
use axgeom::AxisTrait;
use compt::CTreeIterator;
use is_sorted::IsSorted;


pub fn are_invariants_met<A:AxisTrait,N,T:HasAabb>(tree:&DynTree<A,N,T>)->Result<(),()>{
    inner(tree.get_axis(),tree.get_iter().with_depth(compt::Depth(0)))
}

fn a_bot_has_value<N:NumTrait>(it:impl Iterator<Item=N>,val:N)->bool{
    for b in it{
        if b==val{
            return true;
        }
    }
    return false;
}

fn inner<A:AxisTrait,N,T:HasAabb>(axis:A,iter:compt::LevelIter<NdIter<N,T>>)->Result<(),()>{
    
    macro_rules! assert2{
        ($bla:expr)=>{
            if !$bla{
                return Err(())
            }
        }
    }

    let ((depth,nn),rest)=iter.next();

    let axis_next=axis.next();
    

    assert2!(nn.range.iter().is_sorted_by(|a,b|{
        a.get().get_range(axis_next).left.cmp(&b.get().get_range(axis_next).left)
    }));

    
    match rest{
        Some((extra,left,right))=>{
            match extra{
                Some(compt)=>{
                    for bot in nn.range.iter(){
                        assert2!(bot.get().get_range(axis).contains(compt.div));
                    }

                    for bot in nn.range.iter(){
                        assert2!(compt.cont.contains_range(bot.get().get_range(axis)));
                    } 

                    assert2!(a_bot_has_value(nn.range.iter().map(|b|b.get().get_range(axis).left),compt.div));
                    assert2!(a_bot_has_value(nn.range.iter().map(|b|b.get().get_range(axis).left),compt.cont.left));
                    assert2!(a_bot_has_value(nn.range.iter().map(|b|b.get().get_range(axis).right),compt.cont.right));

                    inner(axis_next,left);
                    inner(axis_next,right);

                },
                None=>{
                    assert2!(nn.range.is_empty());
                    for ((_depth,n),e) in left.dfs_preorder_iter().chain(right.dfs_preorder_iter()){
                        assert2!(e.is_none());
                        assert2!(n.range.is_empty());
                    }
                }
            }

        },
        None=>{
        }
    }
    Ok(())
}
