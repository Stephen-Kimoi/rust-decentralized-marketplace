#[macro_use] 
extern crate serde;
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable}; 
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager,VirtualMemory}; 
use candid::{Decode, Encode, Principal};  
use std::{borrow::Cow, cell::RefCell}; 

type Memory = VirtualMemory<DefaultMemoryImpl>; 
type IdCell = Cell<u64, Memory>; 

// Items Struct 
#[derive(candid::CandidType, Serialize, Deserialize, Default, Clone)] 
struct Item {
    id: u64, 
    name: String, 
    description: String
} 

// Serializing the items for storage and transmission 
impl Storable for Item {
   fn to_bytes(&self) -> Cow<[u8]> {
       Cow::Owned(Encode!(self).unwrap())
   }     

   fn from_bytes(bytes: Cow<[u8]>) -> Self {
       Decode!(bytes.as_ref(), Self).unwrap()
   }
}

