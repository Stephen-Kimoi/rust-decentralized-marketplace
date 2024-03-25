#[macro_use] 
extern crate serde;
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable}; 
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager,VirtualMemory}; 
use candid::{Decode, Encode, Principal};
use serde::de::value::Error;  
use std::{borrow::Cow, cell::RefCell}; 
use ic_cdk::{query, update}; 

type Memory = VirtualMemory<DefaultMemoryImpl>; 
type IdCell = Cell<u64, Memory>; 

// Items Struct 
#[derive(candid::CandidType, Serialize, Deserialize, Default, Clone)] 
struct Item {
    id: u64, 
    name: String, 
    description: String, 
    amount: u64
} 

// Serializing & Deserializing the items for storage and transmission 
impl Storable for Item {
   fn to_bytes(&self) -> Cow<[u8]> {
       Cow::Owned(Encode!(self).unwrap())
   }     

   fn from_bytes(bytes: Cow<[u8]>) -> Self {
       Decode!(bytes.as_ref(), Self).unwrap()
   }
}

impl BoundedStorable for Item {
    const MAX_SIZE: u32 = 1024; 
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {

    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    ); 
    
    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );
    
    static ITEM_STORAGE: RefCell<StableBTreeMap<u64, Item, Memory>> = 
    RefCell::new(StableBTreeMap::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    )); 
}

// #[derive(candid::CandidType, Deserialize, Serialize)]
// enum Error {
//     NotFound { msg: String },
// }

#[update] 
fn list_item(item: Item) -> Option<Item> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get(); 
            counter.borrow_mut().set(current_value + 1)
        }) 
        .expect("Cannot increament ID counter"); 

    let item = Item {
        id, 
        name: item.name, 
        description: item.description, 
        amount: item.amount
    }; 
    
    ITEM_STORAGE.with(|service| service.borrow_mut().insert(id, item.clone())); 
    Some(item)

}

#[query] 
fn return_items() -> Vec<Item> {
    ITEM_STORAGE.with(|service| service.borrow().iter().map(|(_, item) | item.clone()).collect())
}

// Export Candid interface
ic_cdk::export_candid!();