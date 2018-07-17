/*
        let c=self.get_iter().with_depth(Depth(0));


        fn recc<'a,A:AxisTrait,N:'a,T:HasAabb+'a>(axis:A,cc:LevelIter<NdIter<N,T>>){
            let ((_depth,nn),rest)=cc.next();

            match rest{
                Some((extra,left,right))=>{

                    let &FullComp{div,cont:_}=match extra{
                        Some(g)=>g,
                        None=>unimplemented!("FINISH THIS")
                    };

                    for b in &nn.range{
                        let r=b.get().as_axis().get(axis);
                        assert!(r.left<=div && r.right>=div);
                    }        


                    recc(axis.next(),left);
                    recc(axis.next(),right);
                },
                None=>{

                }
            }
        }
        recc(self.tree.get_axis(),c);
        */
        /*
        fn assert_correctness(&self,tree:&KdTree,botman:&BotMan)->bool{
            for (level,axis) in kd_axis::AxisIter::with_axis(tree.tree.get_level_iter()) {
                if level.get_depth()!=tree.tree.get_height()-1{
                    for n in level.iter(){
                        let no=tree.tree.get_node(n);
                        let cont_box=&no.container_box;// no.get_divider_box(&botman.prop,axis);

                        let arr=&tree.collision_botids[no.container.get_range().as_int_range()];
                        for b in arr{
                            let bot=botman.cont.get_bot(*b);
                            let circle=&botman.as_circle(bot);
                            assert!(cont_box.contains_circle(circle),"{:?}\n{:?}\n{:?}\n{:?}",no,(level,axis),cont_box,circle);
                        }
                    }
                }
                
            }
             

            let arr=&tree.collision_botids[tree.no_fit.end.0..];
            let mut cols=0;
            for (i, el1) in arr.iter().enumerate() {
                for el2 in arr[i + 1..].iter() {
                    let bb=(*el1,*el2);
                    let bots = botman.cont.get_bbotpair(bb);

                    match bot::is_colliding(&botman.prop, bots) {
                        Some(_) => {
                            cols+=1;
                        }
                        None => {
                        }
                    }
                }
            }

            let mut cls=0;
            for k in self.binner_helps.iter(){
                cls+=k.cols_found.len();
            }

            let lookup=|a:(BotIndex, BotIndex)|{
                for k in self.binner_helps.iter(){
                    for j in k.cols_found.iter(){
                        let aa=( (j.inds.0).0 ,(j.inds.1).0);
                        let bb=((a.0).0,(a.1).0);
                        if aa.0==bb.0 &&aa.1==bb.1{
                            return true;
                        }
                        if aa.0==bb.1 && aa.1==bb.0{
                            return true;
                        }
                    }
                }
                false            
            };
            if cols!=cls{
                println!("Cols fail! num collision exp:{:?},  calculated:{:?}",cols,cls);

                for (i, el1) in arr.iter().enumerate() {
                    for el2 in arr[i + 1..].iter() {
                        let bb=(*el1,*el2);
                        let bots = botman.cont.get_bbotpair(bb);

                        match bot::is_colliding(&botman.prop, bots) {
                            Some(_) => {
                                if !lookup(bb){
                                    println!("Couldnt find {:?}",(bb,bots));

                                    println!("in node:{:?}",(lookup_in_tree(tree,bb.0),lookup_in_tree(tree,bb.1)));
                                    let n1=lookup_in_tree(tree,bb.0).unwrap();
                                    let n2=lookup_in_tree(tree,bb.1).unwrap();
                                    let no1=tree.tree.get_node(n1);
                                    let no2=tree.tree.get_node(n2);
                                    
                                    println!("INTERSECTS={:?}",no1.cont.border.intersects_rect(&no2.cont.border));

                                }
                            }
                            None => {
                            }
                        }
                    }
                }
                assert!(false);
            }
            
            fn lookup_in_tree(tree:&BaseTree,b:BotIndex)->Option<NodeIndex>{
                for level in tree.tree.get_level_iter(){
                    for nodeid in level.iter().rev() {
                        
                        let n = tree.tree.get_node(nodeid);
                    
                        let k=n.container.get_range().as_int_range();

                        let arr=&tree.collision_botids[k];
                        for i in arr{
                            if b.0==i.0{
                                return Some(nodeid);
                            }
                        }
                    }
                }
                return None
            }
            true
        }*/
        /*
        //Note this doesnt check all invariants.
        //e.g. doesnt check that every bot is in the tree only once.
        fn assert_invariant<T:SweepTrait>(d:&DinoTree2<T>){
            
            let level=d.0.get_level_desc();
            let ll=compt::LevelIter::new(d.0.get_iter(),level);
            use compt::CTreeIterator;
            for (level,node) in ll.dfs_preorder_iter(){
               
               //println!("level={:?}",level.get_depth());
               if level.get_depth()%2==0{
                  oned::is_sorted::<A::Next,_>(&node.range);


                  let kk=node.container_box;
                  for a in node.range.iter(){
                     let (p1,p2)=(
                          a.get().0.get().get_range2::<A>().left(),
                          a.get().0.get().get_range2::<A>().right());
                      assert!(kk.left()<=p1);
                      assert!(p2<=kk.right());
                  }
               }else{
                  oned::is_sorted::<A,_>(&node.range);
                  
                  let kk=node.container_box;
                  for a in node.range.iter(){
                     let (p1,p2)=(
                          a.get().0.get().get_range2::<A::Next>().left(),
                          a.get().0.get().get_range2::<A::Next>().right());
                      assert!(kk.left()<=p1);
                      assert!(p2<=kk.right());
                  }
               }
            }       
            
        }
        */