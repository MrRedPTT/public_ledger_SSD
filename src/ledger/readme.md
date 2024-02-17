# Ledger Documentation

## Blockchain

### new
- **definition:** `new(is_miner:bool) -> Blockchain`

### add_block
- **definition:** `add_block(b:Block) -> bool`
- **outputs:**
    returns true if the block is successfully added
- **description:**
    adds a block to the blockchain,
    if the `prev_hash` of b is not the hash of the head of the blockchain then
    the client will ask the network for missing block(s)
- **status:** **not fully implemented**
    - missing getting other packages from network
    - verification is also not fully done

### add_transaction
- **definition:** `add_transaction(t:Transaction)`
- **description:**
    only important to miners,
    adds a transaction to a temporary block
    when the block is full it will be mined

### get_head
- **definition:** `get_head() -> Block`
- **description:**
    returns the most recent Block of the blockchain


## Block

### new
- **definition:** ``` new(index: usize, 
                   prev_hash: String, 
                   difficulty: usize, 
                   miner_id: String,
                   miner_reward:f64) -> Block  ```
- **description:**
    creates a new block with a single transaction (the miner reward)

### mine
- **definition:** ` mine() -> bool`
- **outputs:**
    returns true when the block is mined with success
- **description:**
    mines the block
    

### add_transaction
- **definition:** ` add_transaction(t:Transaction) -> i64`
- **outputs:**
    returns the id of the transaction
- **description:**    
    adds a transaction to the block
    if the number of transactions exceeds the max ammount and the client is a miner 
    then the block is mined

### check_hash
- **definition:** `check_hash() -> bool`
- **description:**
    checks if the hash of the block is correct with reference to its dificulty

### calculate_hash
- **definition:** `calculate_hash() -> String`
- **outputs:** the hash of the block

### transactions_to_string
- **definition:** ` transactions_to_string() -> String`
- **outputs:** a string of all the transactions inside this block 

## Transactions

### new
- **definition:** `new(amount_in: f64, from: String, amount_out: f64, to: String ) -> Transaction`

### to_string
- **definition:** `to_string() -> String`
- **outputs:** returns the transaction in string form
