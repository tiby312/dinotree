use axgeom::AxisTrait;
use compt::Visitor;
use is_sorted::IsSorted;
use crate::inner_prelude::*;

use crate::tree::Vistr;


///Returns Ok, then this tree's invariants are being met.
///Should always return true, unless the user corrupts the trees memory
///or if the contract of the HasAabb trait are not upheld.
pub fn are_invariants_met<A:AxisTrait,N,T:HasAabb>(tree:DinoTreeRef<A,N,T>)->Result<(),()> where T::Num : std::fmt::Debug{
    let axis=tree.axis();

    inner(axis,tree.vistr().with_depth(compt::Depth(0)))
}

fn a_bot_has_value<N:NumTrait>(it:impl Iterator<Item=N>,val:N)->bool{
    for b in it{
        if b==val{
            return true;
        }
    }
    false
}

fn inner<A:AxisTrait,N,T:HasAabb>(axis:A,iter:compt::LevelIter<Vistr<N,T>>)->Result<(),()> where T::Num : std::fmt::Debug{
    
    macro_rules! assert2{
        ($bla:expr)=>{
            if !$bla{
                return Err(())
            }
        }
    }

    let ((_depth,nn),rest)=iter.next();

    let axis_next=axis.next();

    let f=|a:&&T,b:&&T|->std::cmp::Ordering{
        a.get().get_range(axis_next).left.cmp(&b.get().get_range(axis_next).left)
    };
    assert2!(nn.range.iter().is_sorted_by(f));
    
    if let Some((extra,left,right))=rest{
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

                
                inner(axis_next,left)?;
                inner(axis_next,right)?;

            },
            None=>{
                assert2!(nn.range.is_empty());
                
                for ((_depth,n),e) in left.dfs_preorder_iter().chain(right.dfs_preorder_iter()){
                    if let Some(cc)=e{
                        assert2!(cc.is_none());
                    }
                    assert2!(n.range.is_empty());
                }
            }
        }

    
    }
    Ok(())
}
