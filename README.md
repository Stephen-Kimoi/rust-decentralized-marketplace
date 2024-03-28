# Decentralized Marketplace 

This is a decentralzied marketplace canister smart contract deployed on the internet computer protocol. 

The platform utilizes ICP's robustness and efficiency to ensure secure and reliable operations 

The marketplace contains the following main functionalities: 
1. Registering of sellers and buyers 
2. Listing of items in the marketplace 
3. Buying of items from the marketplace 
4. Updating items listed in the marketplace 
5. Deleting items that you no longer want to sell in the marketplace 
6. Getting the specific sellers' details of the items listed in the marketplace 

## Main Funcionalities: 
### 1. User Management 
#### 1.1 Register User: 
_Functionality:_ Allows users to register with the platform, providing essential information such as username, email, and role.
```
register_user()
```

##### Usage: 
- new_user: New user details including username, email, and role.
- Returns a 'User' struct upon successful registration or an error if any field is empty or if the user already exists.

#### 1.2 Get sellers and their items: 
_Functionality:_ Retrieves sellers along with the items they have listed.
```
get_sellers_and_items() 
```

##### Usage: 
- Returns a vector containing tuples of seller information (username, email, principal_id) and a vector of items listed by the seller. 

### 2. Item Management: 
#### 2.1 List Item: 
_Functionality:_ Allows sellers to list items for sale, providing item details such as name, description, and amount.

```
list_item()
```
##### Usage: 
- new_item: Takes in details of the new item including name, description, and amount.
- Returns the listed Item upon successful listing or an error if any field is empty or if the seller is not registered.

#### 2.2 Return Items: 
_Functionality:_ Retrieves all items listed on the platform.
```
return_items()  
```

##### Usage: 
- Returns a vector containing all items listed on the platform. 

#### 2.3 Delete Item: 
_Functionality:_ Allows item owners to delete their listed items.

```
delete_item()
``` 
##### Usage:  
- id: takes ID of the item to be deleted.
- Returns success upon item deletion or an error if the item is not found or if the caller is not the owner of the item.

#### 2.4 Update Item: 
_Functionality:_ Enables item owners to update the details of their listed items.

##### Usage:  
- id: takes in ID of the item to be updated.
- new_name, new_description, new_amount: Updated details of the item.
- Returns success upon item update or an error if the item is not found or if the caller is not the owner of the item.

## Other functionalities: 
- Error Handling: The platform employs an error enum Error to handle various error scenarios such as field emptiness, unauthorized access, item not found, etc.

- Serialization and Deserialization: The platform provides serialization and deserialization implementations for seamless storage and transmission of User and Item structs.

- Memory Management: Utilizes a MemoryManager for efficient memory handling, ensuring optimal performance within the distributed environment.

- Thread Safety: The platform employs thread-local storage for managing critical resources such as memory, item counters, and user data, ensuring thread safety and concurrent access.

## Conclusion: 
Feel free to create a pull request if you have any suggestions/changes




