#[macro_use] 
extern crate serde;
// use candid::types::principal;
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable}; 
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager,VirtualMemory}; 
use candid::{CandidType, Decode, Encode, Principal };
// use std::default;
// use std::collections::BTreeMap;
// use serde::de::value::Error;  
use std::{borrow::Cow, cell::RefCell}; 
use ic_cdk::{pre_upgrade, query, update}; 
use std::collections::BTreeMap; 

type Memory = VirtualMemory<DefaultMemoryImpl>; 
type IdCell = Cell<u64, Memory>; 
// type ItemStore = BTreeMap<Principal, Item>; 

// User Roles 
#[derive(CandidType, Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)] 
enum UserRole {
    #[default] 
    Empty, 
    Seller, 
    Buyer
}

// User struct
#[derive(CandidType, Serialize, Deserialize, Clone)] 
struct User {
    id: u64, 
    username: String, 
    email: String, 
    principal_id: Principal, 
    role: UserRole, 
}

impl Default for User {
    fn default() -> Self {
        Self {
         id: 0, 
         username: String::new(), 
         email: String::new(), 
         principal_id: Principal::anonymous(),
         role: UserRole::Empty, 
        }
    }   
}

// New user struct 
#[derive(CandidType, Serialize, Deserialize)] 
struct NewUser {
    username: String, 
    email: String, 
    role: UserRole
} 

// Items Struct 
#[derive(candid::CandidType, Serialize, Deserialize, Clone )] 
struct Item {
    id: u64, 
    name: String, 
    description: String, 
    amount: u64,
    principal_id: Principal, 
    sold: bool
} 

impl Default for Item {
   fn default() -> Self {
       Self {
        id: 0, 
        name: String::new(), 
        description: String::new(), 
        amount: 0, 
        principal_id: Principal::anonymous(), 
        sold: false
       }
   }   
}

// New Item struct 
#[derive(candid::CandidType, Serialize, Deserialize, Default, Clone)] 
struct NewItem {
    name: String, 
    description: String, 
    amount: u64
} 

// Serializing & Deserializing for storage and transmission 
impl Storable for Item {
   fn to_bytes(&self) -> Cow<[u8]> {
       Cow::Owned(Encode!(self).unwrap())
   }     

   fn from_bytes(bytes: Cow<[u8]>) -> Self {
       Decode!(bytes.as_ref(), Self).unwrap()
   }
}

impl Storable for User {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Item {
    const MAX_SIZE: u32 = 1024; 
    const IS_FIXED_SIZE: bool = false;
}

impl BoundedStorable for User {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    ); 
    
    static ITEM_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );
    
    static ITEM_STORAGE: RefCell<StableBTreeMap<u64, Item, Memory>> = 
    RefCell::new(StableBTreeMap::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    )); 

    static USER_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))), 0)
            .expect("Cannot create a counter")
    );

    static USERS: RefCell<BTreeMap<Principal, User>> = RefCell::default(); 

    // static ITEMS: RefCell<ItemStore> = RefCell::default(); 
}

// For erasing the canister's data when re-deploying
#[pre_upgrade]
fn pre_upgrade() {
    ITEM_STORAGE.with(|service| {
        *service.borrow_mut() = StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        );
    });
}

// For errors 
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    FieldEmpty { msg: String }, 
    Sold { msg: String }, 
    Unauthorized { msg: String }, 
    UserExists { msg: String }, 
    UserNotRegistered { msg: String }
}

// Function for registering users 
#[update]
fn register_user(new_user: NewUser) -> Result<User, Error> {
    if new_user.email.is_empty() || new_user.username.is_empty() || new_user.role == UserRole::Empty {
        return Err(Error::FieldEmpty { msg: format!("Kindly ensure all fields aren't empty") })
    } 

    let id = USER_COUNTER
    .with(|counter| {
        let current_value = *counter.borrow().get();
        counter.borrow_mut().set(current_value + 1)
    })
    .expect("cannot increment id counter");

    USERS.with(|users| {
       let mut users_borrowed = users.borrow_mut(); 
       let principal_id_of_caller = ic_cdk::caller();  

       if users_borrowed.contains_key(&principal_id_of_caller) {
        return Err(Error::UserExists { msg: format!("User with this Principal ID already exists!") })
       }

       let user = User {
            id, 
            username: new_user.username, 
            email: new_user.email, 
            principal_id: principal_id_of_caller, 
            role: new_user.role 
        }; 

        users_borrowed.insert(principal_id_of_caller, user.clone()); 
        Ok(user)
    }) 

}

// Function for listing item
#[update] 
fn list_item(new_item: NewItem) -> Result<Item, Error> {

    if new_item.name.is_empty() || new_item.description.is_empty() || new_item.amount == 0 {
        return Err(Error::FieldEmpty { msg: "Fill in all required fields!".to_string(), }); 
    }

    let seller_principal_id = ic_cdk::caller(); 

    // Checking if seller is registered 
    let is_seller_registered = USERS.with(|users| {
        users.borrow().contains_key(&seller_principal_id)
    }); 

    if !is_seller_registered {
        return Err(Error::UserNotRegistered { msg: format!("Seller has not registered!") })
    }

    let id = ITEM_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get(); 
            counter.borrow_mut().set(current_value + 1)
        }) 
        .expect("Cannot increament ID counter"); 

    let item = Item {
        id, 
        name: new_item.name, 
        description: new_item.description, 
        amount: new_item.amount,  
        principal_id: seller_principal_id,
        sold: false
    }; 
    
    ITEM_STORAGE.with(|service| service.borrow_mut().insert(id, item.clone())); 
    Ok(item)
}

// Function for returning the items listed 
#[query] 
fn return_items() -> Vec<Item> {
    ITEM_STORAGE.with(|service| service.borrow().iter().map(|(_, item) | item.clone()).collect())
}

// Function for deleting the listed item 
#[update]
fn delete_item(id: u64) -> Result<(), Error> {
    let caller = ic_cdk::caller(); 
    ITEM_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut(); 
        if let Some(item) = storage.get(&id) {
            if item.principal_id == caller {
                storage.remove(&id); 
                Ok(())
            } else {
                Err(Error::Unauthorized { msg: format!("Caller is not the owner of item with ID {}", id), })
            }
        } else {
            Err(Error::NotFound { msg: format!("Item with ID {} is not found!", id), })
        }
    })
}

// Function for updating listed item 
#[update]
fn update_item(id: u64, new_name: String, new_description: String, new_amount: u64) -> Result<(), Error> {
   let caller = ic_cdk::caller(); 
   
    match _get_item(&id) {
       Some(mut item) => {
        if item.principal_id == caller {
            item.name = new_name; 
            item.description = new_description; 
            item.amount = new_amount; 

            ITEM_STORAGE.with(|service| service.borrow_mut().insert(id, item.clone())); 
            Ok(())
        } else {
            Err(Error::Unauthorized { msg: format!("Caller is not owner of item with ID {}", id) })
        }
       }
       None => Err(Error::NotFound { msg: format!("Item with ID {} could not be found!", id) })
    }
    
}

// Function for returning seller and the items they've listed 
#[query] 
fn get_sellers_and_items() -> Vec<(String, String, Principal, Vec<Item>)> {
    let mut result = Vec::new(); 

    USERS.with(|users| {
        let users_borrowed = users.borrow(); 

        for (principal_id, user) in users_borrowed.iter() {
            let items = ITEM_STORAGE.with(|items| {
                let items_borrowed = items.borrow(); 

                items_borrowed.iter() 
                  .filter(|(_, item)| item.principal_id == *principal_id) 
                  .map(|(_, item)| item.clone())
                  .collect::<Vec<Item>>()
            }); 

            result.push((user.username.clone(), user.email.clone(), *principal_id, items)); 
        }
    }); 

    result 
}

// Helper function to get item ID 
fn _get_item(item_id: &u64) -> Option<Item> {
    ITEM_STORAGE.with(|service| service.borrow().get(item_id))
}


// Export Candid interface
ic_cdk::export_candid!();