  ///Compute a metric to determine how healthy the tree is based on how many bots
    ///live in higher nodes verses lower nodes. Ideally all bots would live in leaves.
    pub fn compute_tree_health(&self)->f64{
        
        fn recc<N,T:HasAabb>(a:LevelIter<NdIter<N,T>>,counter:&mut usize,height:usize){
            let ((depth,nn),next)=a.next();
            match next{
                Some((_extra,left,right))=>{
                    *counter+=nn.range.len()*(height-1-depth.0);
                    recc(left,counter,height);
                    recc(right,counter,height);
                },
                None=>{
                }
            }
        }
        let height=self.get_height();
        let mut counter=0;
        recc(self.get_iter().with_depth(Depth(0)),&mut counter,height);

        unimplemented!("Not yet implemented");
    }