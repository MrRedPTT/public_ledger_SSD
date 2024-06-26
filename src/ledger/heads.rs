use crate::ledger::block::*;

#[derive(Debug,Clone)]
pub struct Heads {
    list: Vec<Vec<Block>>,
    max_confirms:  usize,
}

impl Heads {
    pub fn new(v:Vec<Block>, max_confirms: usize) -> Heads {
        Heads {
            list: vec![v],
            max_confirms
        }
    }
    pub fn num(&self) -> usize {
        return self.list.len()
    }

    pub fn get_main(&self) -> Vec<Block>{
        return self.list[0].clone();
    }

    pub fn add_block(&mut self, b:Block) -> bool{
        // check each of the 
        for head in &mut self.list{
            // @ beginning of a head
            let mut index = head.len()-1;
            if head[index].hash == b.prev_hash {
                head.iter_mut().for_each(|block| block.add_confirmation());
                head.push(b.clone());
                return true;
            }

            // @ tail of head
            loop {
                index-=1;

                if head[index].hash == b.prev_hash {
                    let mut nh = vec![];
                    for i in 0..index+1 {
                        println!("{:#}",i);
                        nh.push(head[i].clone());
                    }
                    nh.push(b.clone());

                    self.list.push(nh);
                    self.reorder();
                    return true;
                }
                if index == 0 {
                    break;
                }
            }
        }
        return false;
    }
    
    pub fn add_head(&mut self ,v: Vec<Block>){
        self.list.push(v);
    }

    pub fn reorder(&mut self){
        self.list.sort_by(|a, b| b.len().cmp(&a.len()));
    }

    pub fn prune(&mut self, hash: String){
        let mut i :i64 = 0;
        loop {
            if  i as usize >= self.list.len() {
                break;
            }
            if  self.list[i as usize][0].prev_hash == hash {
                self.list.remove(i as usize);
                i-=1;
            }
            i+=1;
        }
    }

    pub fn get_confirmed(&mut self) -> Option<Block>{
        for i in 0..self.list.len(){
            if self.list[i].len() > self.max_confirms {
                return Some(self.list[i].remove(0))
            }
        };               
        return None
    }

    pub fn can_add_block(&self, b: Block) -> bool{
        // check each of the 
        for head in &self.list{
            // @ beginning of a head
            let mut index = head.len()-1;
            if head[index].hash == b.prev_hash {
                return true;
            }

            // @ tail of head
            loop {
                if index <= 0 {
                    break;
                }

                index-=1;

                if head[index].hash == b.prev_hash {
                    return true;
                }
            }
        }
        return false;
    }

    pub fn get_block_by_hash(&self, hash: String) -> Option<Block> {
        
        for head in self.list.iter() {
            for block in head.iter().rev() {
                if block.hash ==  hash{
                    return Some(block.clone());
                }
            }
        }
        
        return None;
    }
}
