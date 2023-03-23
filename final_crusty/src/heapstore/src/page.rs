use common::ids::{PageId, SlotId};
use common::PAGE_SIZE;
use std::convert::TryInto;
use std::mem;
 
 
/// The struct for a page. Note this can hold more elements/meta data when created,
/// but it must be able to be packed/serialized/marshalled into the data array of size
/// PAGE_SIZE. In the header, you are allowed to allocate 8 bytes for general page metadata and
/// 6 bytes per value/entry/slot stored. For example a page that has stored 3 values, can use
/// up to 8+3*6=26 bytes, leaving the rest (PAGE_SIZE-26 for data) when serialized.
/// You do not need reclaim header information for a value inserted (eg 6 bytes per value ever inserted)
/// The rest must filled as much as possible to hold values.
 
//Page design
//header - 8 bytes of general meta data
// 6 bytes of metadata for value one
// 6 bytes of metadata for value two
//free space
//value one
//value two
 
 
//Declare the page struct
pub(crate) struct Page {
   /// The data for data
   data: [u8; PAGE_SIZE], //shouldn't is be page size - 8?
   header: Header,
   metadata: Vec<Metadata>, //have a vector of metadata
}
 
//Declare Header struct which is 8 bytes
pub struct Header {
   p_id: PageId,  // 2 bytes
   upd_offset: u16, //2bytes
   n_records: u16,
   n_deletes: u16,
   deleted_slots: Vec<u16>, //2 bytes * n
  
}
//4 bytes long
pub struct Metadata {
   record_size: u16,
   offset: u16 ,
}
 
/// The functions required for page
impl Page {
   /// Create a new page
   pub fn new(page_id: PageId) -> Self {
       //declare the vectors
       let mut deleted_s = Vec::new();
       let mut m_vector: Vec<Metadata> = Vec::new();
 
       //declare the header
       let page_header = Header {
           p_id: page_id,
           upd_offset: (PAGE_SIZE - 1) as u16,
           n_records: 0,
           n_deletes: 0,
           deleted_slots: deleted_s,
       };
       //return the page
       Page{
           data: [0;PAGE_SIZE],
           header: page_header,
           metadata: m_vector,
       }
      
   }
 
   /// Return the page id for a page
   //Page -> page_id
   pub fn get_page_id(&self) -> PageId {
       let pg_id: u16 = self.header.p_id;
       pg_id
 
   }
   /// Attempts to add a new value to this page if there is space available.
   /// Returns Some(SlotId) if it was inserted or None if there was not enough space.
   /// Note that where the bytes are stored in the page does not matter (heap), but it
   /// should not change the slotId for any existing value. This means that
   /// bytes in the page may not follow the slot order.
   /// If a slot is deleted you should replace the slotId on the next insert.
   ///
   /// HINT: You can copy/clone bytes into a slice using the following function.
   /// They must have the same size.
   /// self.data[X..y].clone_from_slice(&bytes);
 
 
 
/*
1) check if offset is at the end
2) else I check if i have any available deleted slots I can overwrite
3) else I check if I have enough free space to add the value in
4) else I return none
 
for deleting values
assign the smallest available slotId.
 
*/
 
   pub fn overwrite_helper(&mut self, bytes: &[u8]) -> Option<SlotId> {
       println!("OVERWRITE FUNCTION");
      
       let bytes_size = bytes.len();
       println!("bytes size to insert is {}",bytes_size);
       //checks if there is deleted space where you can ovewrite
       let mut counter: usize = 0;
       //let mut able_to_fit = 0;
 
       //if we can successfully fit the value into one of the deleted value holes in one of the vectors
       for i in &self.header.deleted_slots{
           //let i = i as usize;
          
           //retrieve current slot id and corresponding record size in bytes
           let slot_id_v: usize = self.header.deleted_slots[counter].into();
           println!("delete slot id to be overwritten is {}",slot_id_v);
           let space_available = self.metadata[slot_id_v].record_size;
           println!("space available is {}",space_available);
 
           //check to see if we can fit in the record in the current available space
           if bytes_size <= space_available.into(){
               println!("entered space available");
               //add in the bytes into that space
               let strt_index: usize = self.metadata[slot_id_v].offset.into(); //retrieve the offset
               let end_index: usize = strt_index + bytes_size as usize; //or should I make the index the next offset to keep it the same?
               self.data[(strt_index+1)..=end_index].clone_from_slice(&bytes);
               let new_metadata = Metadata{
                   record_size: bytes_size as u16,
                   offset: strt_index as u16,
               };
      
               //update the metadata
               self.metadata[slot_id_v] = new_metadata;
               println!("slot id to be removed from slot id deleted vector is {}",slot_id_v);
               let slot_id_u16 = slot_id_v as u16;
               self.header.deleted_slots.retain(|&k|k != slot_id_u16);
               self.header.n_deletes -= 1;
 
               //able_to_fit = 1;
               println!("about to return is {}",slot_id_u16);
               return Some(slot_id_u16);
  
           }
           else {
               println!("not enough space to fit record");
               counter += 1;
               continue;
              
            //go to next iteration a check next available space
           }
          
 
       }
       println!("was not able to fit");
       //if was not able to fit
       let use_slot_id = self.header.deleted_slots[0];
       let new_offset = self.header.upd_offset - bytes_size as u16;
       let new_metadata = Metadata{
           record_size: bytes_size as u16,
           offset: new_offset as u16,
       };
       self.metadata[use_slot_id as usize] = new_metadata;
       let ret_slotid = use_slot_id as u16;
       self.header.deleted_slots.retain(|&k|k != ret_slotid );
       self.header.n_deletes -= 1;
       println!("lenth of deleted_slots is {}",self.header.deleted_slots.len());
       return Some(ret_slotid);
   }
 
   pub fn add_value(&mut self, bytes: &[u8]) -> Option<SlotId> {
       println!("ADD VALUE FUNCTION");
       let bytes_size = bytes.len();
       println!("start offset {}",self.header.upd_offset);
       println!("the length of bytes are {}",bytes_size);
       let size_u16 = bytes_size as u16;
       let len_slotid_vec = self.metadata.len();
       println!("curr slot id is {}", len_slotid_vec);
       let page_size_index = PAGE_SIZE - 1;
       println!("Page size is {}",PAGE_SIZE);
       println!("Page size index is {}",page_size_index);
 
       //let over_write_global = self.overwrite_helper(bytes);
 
       let mut flag = 0; //for deletion
       //println!("return overwrite  value outside of else if {:?}", over_write_global);
       //check if offset is at the end
       if self.header.upd_offset == page_size_index as u16
       {
           println!{"entered edge"}
           //add in slot id = 0 -> first slot id
           //self.header.slot_id_vec.push(0);
           self.header.upd_offset = page_size_index as u16 - size_u16;
           let strt_index = self.header.upd_offset as usize;
           println!("start index is {}",strt_index+1);
           println!("end index is {}",page_size_index);
           //add in the bytes
           self.data[(strt_index+1)..=page_size_index].clone_from_slice(&bytes);
           //declare new metadata
           let new_metadata = Metadata{
               record_size: bytes_size as u16,
               offset: self.header.upd_offset as u16,
 
           };
           //add in metadata and update the free space
           self.metadata.push(new_metadata);
           self.header.n_records += 1;
           println!("new offset is {}",self.header.upd_offset);
           return Some(0);
       } 
       //adding a value to a deleted spot so your not calling delete your just overwriting
       //if self.deleted.slots != 0
       //overwrite and re assign any deleted slot values
       else if self.header.deleted_slots.len() != 0
       {
           let over_write_global = self.overwrite_helper(bytes);
           println!("In add value overwrite");
           println!("return overwrite  value {:?}", over_write_global);
          return over_write_global;
       }
      
       //check to see if there is available deleted space for where we could overwrite our bytes
        else if bytes_size <= self.get_largest_free_contiguous_space()
       {
               println!{"entered contig"};
       //slot id will equal length of slot id vector since we start indexing at 0
               let ret_slotid = len_slotid_vec;
               //self.header.slot_id_vec.push(len_slotid_vec.try_into().unwrap());
 
       //add in the bytes in data
               let end_index = self.header.upd_offset as usize;
               self.header.upd_offset = self.header.upd_offset as u16 - size_u16;
               println!("new offset is {}",self.header.upd_offset);
               //println!("updated offset is {}",self.header.upd_offset);
               let strt_index = self.header.upd_offset as usize;  //previous offset - bytes size
               //let end_index = (strt_index + bytes_size) as usize;
               println!("start index for add in is {}",strt_index+1);
               println!("end index for add in is {}",end_index);
               self.data[(strt_index+1)..=end_index].clone_from_slice(&bytes);
 
       //declare the new metadata
               let n_metadata = Metadata{
                  record_size: bytes_size as u16,
                  offset: self.header.upd_offset as u16,
               };
 
       //update the amount of free space and offset vec
               self.metadata.push(n_metadata);
               self.header.n_records += 1;
               println!("the number of current records is {}",self.header.n_records);
               let ret_slotid = ret_slotid as u16;
               return Some(ret_slotid);
       } //otherwise we can't add in the value
       else {
           return None;
       }
//panic!("TODO milestone pg");
   }
 
 
   /// Return the bytes for the slotId. If the slotId is not valid then return None
   pub fn get_value(&self, slot_id: SlotId) -> Option<Vec<u8>> {
       println!("GET VALUE FUNCTION");
       println!("value read");
       println!("slot id is {}",slot_id);
       println!("length metadata is {}",self.metadata.len());
       let slotid_usize = slot_id as usize;
       let mut flag = 0;
       let mut counter = 0;
       println!("delete bool is {}",self.check_delete(slot_id));
       for i in &self.metadata{
           if counter == slot_id{
               flag = 1;
               break;
           } else {
               flag = 0;
           }
           counter +=1;
       }
       println!("flag is {}",flag);
       if self.check_delete(slot_id) == true{
           return None;
       }
       else if (flag == 1) && (self.check_delete(slot_id) == false) {
           println!("enter flag");
           //retrieve offset and record size
           let offset = self.metadata[slotid_usize].offset;
           println!("curr slot id is {}",slot_id);
 
           println!("get value offset is{}",offset);
           let record_size = self.metadata[slotid_usize].record_size;
           let strt_index = offset as usize;
           println!("get value start index is{}",strt_index);
           let end_index = strt_index + record_size as usize;
           println!("get value end index is{}",end_index);
           return Some(self.data[(strt_index+1)..=end_index].to_vec());
          
       } else {
           return None;
       }
   }
  
   pub fn check_delete(&self,slot_id: SlotId) -> bool {
       println!("CHECK DELETE FUNCTION");
       let mut counter = 0;
       println!("inputed slot id is {}",slot_id);
       for i in &self.header.deleted_slots{
           println!("DELETED slot id list value is {}",self.header.deleted_slots[counter]);
           if slot_id == self.header.deleted_slots[counter]{
 
               return true;
           }
           else{
               counter += 1;
           }
       }
       println!("not in delete");
       return false;
 
   }
   /// Delete the bytes/slot for the slotId. If the slotId is not valid then return None
   /// The slotId for a deleted slot should be assigned to the next added value
   /// The space for the value should be free to use for a later added value.
   /// HINT: Return Some(()) for a valid delete
   pub fn delete_value(&mut self, slot_id: SlotId) -> Option<()> {
       println!("DELETE VALUE FUNCTION");
       println!("slot id to be deleted is {}",slot_id);
       let slotid_usize = slot_id as usize;
       let mut flag = 0;
       let mut counter = 0;
 
       //first check if slot id is valid
       for i in &self.metadata{
           if counter == slot_id{
               flag = 1;
               break;
           } else {
               flag = 0;
           }
           counter +=1;
       }
       println!("delete flag value is {}",flag);
       if flag == 1 {
         
           println!("ADD TO DELETE VECTOR {}",slot_id);
           self.header.deleted_slots.push(slot_id);
           self.header.n_deletes += 1;
           return Some(());
       } else {
           return None;
       }
       //panic!("TODO milestone pg");
   }
 
   /// Create a new page from the byte array.
   ///
   /// HINT to create a primitive data type from a slice you can use the following
   /// (the example is for a u16 type and the data store in little endian)
   /// u16::from_le_bytes(data[X..Y].try_into().unwrap());
 
   //from_bytes() should create a new Page struct, fill its header correctly and also fill its data correctly.
   // All you have is the bytes and your knowledge of how you structured the bytes during serialization.
   //byte calculation
   pub fn from_bytes(data: &[u8]) -> Self {
       println!("FROM BYTES");
 
       let mut return_vector: Vec<u8> = Vec::new();
       let mut copied_vector = data.to_vec().clone();
      
       //println!("given data in vector form: {:?}",&mut data.to_vec());
       return_vector.append(&mut copied_vector);
       //println!("copy vector is {:?}",return_array);
      
       //create an array
       let mut ret_array: [u8;PAGE_SIZE] = [0;PAGE_SIZE];
       let mut counter = 0;
       for i in &return_vector{
           ret_array[counter] = return_vector[counter];
           counter += 1;
 
       }
       //ret_array[0..=(PAGE_SIZE-1)] = copied_vector[0..=(PAGE_SIZE-1)];
 
       let page_id_v: u16 = u16::from_le_bytes(data[0..=1].try_into().unwrap());
       let upd_offset_v: u16 = u16::from_le_bytes(data[2..=3].try_into().unwrap());
       let n_records_v: u16 = u16::from_le_bytes(data [4..=5].try_into().unwrap());
       let n_deleted_records_v: u16 = u16::from_le_bytes(data[6..=7].try_into().unwrap());
 
       //created the deleted records array
       let mut deleted_records_vec: Vec<u16> = Vec::new();
       let mut i = 0 as usize;
       let mut start_del_index = 8;
       while i < n_deleted_records_v.into() {
           deleted_records_vec.push(u16::from_le_bytes(data[start_del_index..=(start_del_index+1)].try_into().unwrap()));
           start_del_index += 2;
           i += 1;
       }
       println!("the value of start delete index is {}",start_del_index);
       //create the metadata array
       let mut meta_start_index = start_del_index;
       let mut k = 0 as usize;
       let mut m_vector: Vec<Metadata> = Vec::new();
      
       while k < n_records_v.into(){
           let mut record_size_v = u16::from_le_bytes(data[meta_start_index..=(meta_start_index+1)].try_into().unwrap());
           let mut offset_v = u16::from_le_bytes(data[(meta_start_index+2)..=(meta_start_index+3)].try_into().unwrap());
      
           //create metadata
           let curr_metadata = Metadata {
               record_size: record_size_v,
               offset: offset_v,
           };
 
           m_vector.push(curr_metadata);
           meta_start_index += 4;
           k+=1;
 
       }
 
       //now create page header
       let page_header = Header {
           p_id: page_id_v,
           upd_offset: upd_offset_v,
           n_records: n_records_v,
           n_deletes: n_deleted_records_v,
           deleted_slots: deleted_records_vec,
       };
       //now combine everything to make a page
       Page {
           data: ret_array,
           header: page_header,
           metadata: m_vector,
       }
 
       //panic!("TODO milestone pg");
   }
 
   /// Convert a page into bytes. This must be same size as PAGE_SIZE.
   /// We use a Vec<u8> for simplicity here.
   ///
   /// HINT: To convert a vec of bytes using little endian, use
   /// to_le_bytes().to_vec()
   pub fn get_bytes(&self) -> Vec<u8> {
       println!("GET BYTES");
       let mut return_array: Vec<u8> = Vec::new();
       println!("starting array {:?}",return_array);
       //insert page id
       let mut p_id_bytes_v = self.header.p_id.to_le_bytes().to_vec();
       return_array.append(&mut p_id_bytes_v);
       println!("return_array first two bytes is {:?}",return_array);
 
       let mut upd_offset_v = self.header.upd_offset.to_le_bytes().to_vec();
       return_array.append(&mut upd_offset_v);
       println!("return_array after adding upd offset {:?}",return_array);
       //return_array[0..=1].clone_from_slice(&p_id_bytes);
 
       let mut n_records_v = self.header.n_records.to_le_bytes().to_vec();
       return_array.append(&mut n_records_v);
       println!("return_array after adding n_records_v {:?}",return_array);
 
       let mut n_deletes_v = self.header.n_deletes.to_le_bytes().to_vec();
       return_array.append(&mut n_deletes_v);
       println!("return_array after adding n_deletes_v{:?}",return_array);
 
       //take care of deleted slots
       let mut i = 0 as usize;
       while i < self.header.n_deletes.into() {
           let mut deleted_slot_v = self.header.deleted_slots[i].to_le_bytes().to_vec();
           return_array.append(&mut n_records_v);
           i += 1;
       }
 
       //take care of metadata
       let mut k = 0 as usize;
       while k < self.header.n_records.into(){
           let mut record_size_v = self.metadata[k].record_size.to_le_bytes().to_vec();
           return_array.append(&mut record_size_v);
           let mut offset_v = self.metadata[k].offset.to_le_bytes().to_vec();
           return_array.append(&mut offset_v);
 
           k += 1;
       }
     
       //append the bytes representing free space and inserted records
       let header_size = self.get_header_size();
       println!("header_size is {}",header_size);
       println!("size of array before we add in the rest is {:?}",return_array.len());
       let mut stuff_to_append = self.data[header_size..=(PAGE_SIZE-1)].to_vec();
      
       return_array.append(&mut stuff_to_append);
       println!("size of array after we add in the rest is {:?}",return_array.len());
       println!("size of page is {}",PAGE_SIZE);
 
       return_array
 
   }
 
   /// A utility function to determine the size of the header in the page
   /// when serialized/to_bytes.
   /// Will be used by tests. Optional for you to use in your code
   #[allow(dead_code)]
   pub(crate) fn get_header_size(&self) -> usize {
       let n_records = self.metadata.len();
       let n_deletes = self.header.deleted_slots.len();
       let metadata_size = 4*n_records;
       let original_header_size = 8; //bytes
       println!("n records {}",n_records);
       let header_size = original_header_size + metadata_size + 2*n_deletes;
       println!("header size is {}",header_size);
       header_size
       //panic!("TODO milestone pg");
   }
 
   /// A utility function to determine the largest block of free space in the page.
   /// Will be used by tests. Optional for you to use in your code
   #[allow(dead_code)]
   pub(crate) fn get_largest_free_contiguous_space(&self) -> usize {
       let n_records = self.metadata.len();
       //println!("number of records is in get largest {}",n_records);
       let record_size_sum = PAGE_SIZE - (self.header.upd_offset + 1) as usize;
       //println!{"space taken {}",self.header.upd_offset + 1};
       println!("record size sum is {}",record_size_sum);
       let free_space = PAGE_SIZE - self.get_header_size() - record_size_sum as usize;
       println!("free space is {}",free_space);
       free_space
       //panic!("TODO milestone pg");
   }
}
 
/// The (consuming) iterator struct for a page.
/// This should iterate through all valid values of the page.
/// See https://stackoverflow.com/questions/30218886/how-to-implement-iterator-and-intoiterator-for-a-simple-struct
pub struct PageIter {
   //TODO milestone pg
   page: Page,
   index: usize,
   
}
 
/// The implementation of the (consuming) page iterator.
impl Iterator for PageIter {
   type Item = Vec<u8>;
 
   fn next(&mut self) -> Option<Self::Item> {
       let mut curr_index = self.index;
 
       //check the slot id is valid
       if curr_index >= self.page.header.n_records.into(){
           return None;
       }
       //if valid and deleted vector slot is not empty
       else if self.page.header.deleted_slots.len() != 0 {
           println!("FUCK there exists deleted slots");
           let mut i = 0;
           while i < self.page.header.deleted_slots.len(){
               //if curr index is in deleted slots curr_index += 1 and then set i = 0 again
               if curr_index >= self.page.metadata.len() {
                   return None;
               }
               else if curr_index == self.page.header.deleted_slots[i] as usize{
                   curr_index += 1;
                   i = 0;
               }
               else {
                   i += 1
               }
           }
               //let new_v = curr_index;
               self.index = curr_index + 1;
 
               //return the array of bytes
               let offset_v = self.page.metadata[curr_index].offset;
               let size = self.page.metadata[curr_index].record_size as usize;
               let strt_index = offset_v as usize;
               let end_index = strt_index + size;
               return Some(self.page.data[(strt_index+1)..=end_index].to_vec());
       }
       //if valid and not deleted
       else{
           self.index += 1;
 
           //return the array of bytes
           let offset_v = self.page.metadata[curr_index].offset;
           let size = self.page.metadata[curr_index].record_size as usize;
           let strt_index = offset_v as usize;
           let end_index = strt_index + size;
           return Some(self.page.data[(strt_index+1)..=end_index].to_vec());
           }
          
       }
       //panic!("TODO milestone pg");
   }
 
 
/// The implementation of IntoIterator which allows an iterator to be created
/// for a page. This should create the PageIter struct with the appropriate state/metadata
/// on initialization.
impl IntoIterator for Page {
   type Item = Vec<u8>;
   type IntoIter = PageIter;
 
   fn into_iter(self) -> Self::IntoIter {
       PageIter{
           page: self,
           index: 0,
       }
      // panic!("TODO milestone pg");
   }
}
 
#[cfg(test)]
mod tests {
   use super::*;
   use common::testutil::init;
   use common::testutil::*;
   use common::Tuple;
 
   /// Limits how on how many bytes we can use for page metadata / header
   pub const FIXED_HEADER_SIZE: usize = 8;
   pub const HEADER_PER_VAL_SIZE: usize = 6;
 
   #[test]
   fn hs_page_create() {
       init();
       let p = Page::new(0);
       assert_eq!(0, p.get_page_id());
       assert_eq!(
           PAGE_SIZE - p.get_header_size(),
           p.get_largest_free_contiguous_space()
       );
   }
 
   #[test]
   fn hs_page_simple_insert() {
       init();
       let mut p = Page::new(0);
       let tuple = int_vec_to_tuple(vec![0, 1, 2]);
       let tuple_bytes = serde_cbor::to_vec(&tuple).unwrap();
       let byte_len = tuple_bytes.len();
       assert_eq!(Some(0), p.add_value(&tuple_bytes));
       assert_eq!(
           PAGE_SIZE - byte_len - p.get_header_size(),
           p.get_largest_free_contiguous_space()
       );
       let tuple_bytes2 = serde_cbor::to_vec(&tuple).unwrap();
       assert_eq!(Some(1), p.add_value(&tuple_bytes2));
       assert_eq!(
           PAGE_SIZE - p.get_header_size() - byte_len - byte_len,
           p.get_largest_free_contiguous_space()
       );
   }
 
   #[test]
   fn hs_page_space() {
       init();
       let mut p = Page::new(0);
       let size = 10;
       let bytes = get_random_byte_vec(size);
       assert_eq!(10, bytes.len());
       assert_eq!(Some(0), p.add_value(&bytes));
       assert_eq!(
           PAGE_SIZE - p.get_header_size() - size,
           p.get_largest_free_contiguous_space()
       );
       assert_eq!(Some(1), p.add_value(&bytes));
       assert_eq!(
           PAGE_SIZE - p.get_header_size() - size * 2,
           p.get_largest_free_contiguous_space()
       );
       assert_eq!(Some(2), p.add_value(&bytes));
       assert_eq!(
           PAGE_SIZE - p.get_header_size() - size * 3,
           p.get_largest_free_contiguous_space()
       );
   }
 
   #[test]
   fn hs_page_get_value() {
       init();
       let mut p = Page::new(0);
       let tuple = int_vec_to_tuple(vec![0, 1, 2]);
       let tuple_bytes = serde_cbor::to_vec(&tuple).unwrap();
       assert_eq!(Some(0), p.add_value(&tuple_bytes));
       let check_bytes = p.get_value(0).unwrap();
       let val = p.get_value(0);
       println!("Get Value 0: {:?}", val);
       let check_bytes = val.unwrap();
       let check_tuple: Tuple = serde_cbor::from_slice(&check_bytes).unwrap();
       let check_tuple: Tuple = serde_cbor::from_slice(&check_bytes).unwrap();
       assert_eq!(tuple_bytes, check_bytes);
       assert_eq!(tuple, check_tuple);
 
       let tuple2 = int_vec_to_tuple(vec![3, 3, 3]);
       let tuple_bytes2 = serde_cbor::to_vec(&tuple2).unwrap();
       assert_eq!(Some(1), p.add_value(&tuple_bytes2));
       let check_bytes2 = p.get_value(1).unwrap();
       let val = p.get_value(1);
       println!("Get Value 1: {:?}", val);
       let check_bytes = val.unwrap();
       let check_tuple: Tuple = serde_cbor::from_slice(&check_bytes2).unwrap();
       let check_tuple2: Tuple = serde_cbor::from_slice(&check_bytes2).unwrap();
       assert_eq!(tuple_bytes2, check_bytes2);
       assert_eq!(tuple2, check_tuple2);
 
       //Recheck
       let check_bytes2 = p.get_value(1).unwrap();
       let check_tuple2: Tuple = serde_cbor::from_slice(&check_bytes2).unwrap();
       assert_eq!(tuple_bytes2, check_bytes2);
       assert_eq!(tuple2, check_tuple2);
       println!("before recheck");
       let check_bytes = p.get_value(0).unwrap();
       let val = p.get_value(0);
       println!("Get Value 0: {:?}", val);
       let check_bytes = val.unwrap();
       let check_tuple: Tuple = serde_cbor::from_slice(&check_bytes).unwrap();
      
       let check_tuple: Tuple = serde_cbor::from_slice(&check_bytes).unwrap();
       assert_eq!(tuple_bytes, check_bytes);
       assert_eq!(tuple, check_tuple);
 
       //Check that invalid slot gets None
       assert_eq!(None, p.get_value(2));
   }
 
   #[test]
   fn hs_page_header_size_small() {
       init();
       // Testing that the header is no more than 8 bytes for the header, and 6 bytes per value inserted
       let mut p = Page::new(0);
       assert!(p.get_header_size() <= FIXED_HEADER_SIZE);
       let bytes = get_random_byte_vec(10);
       assert_eq!(Some(0), p.add_value(&bytes));
       assert!(p.get_header_size() <= FIXED_HEADER_SIZE + HEADER_PER_VAL_SIZE);
       assert_eq!(Some(1), p.add_value(&bytes));
       assert_eq!(Some(2), p.add_value(&bytes));
       assert_eq!(Some(3), p.add_value(&bytes));
       assert!(p.get_header_size() <= FIXED_HEADER_SIZE + HEADER_PER_VAL_SIZE * 4);
   }
 
   #[test]
   fn hs_page_header_size_full() {
       init();
       // Testing that the header is no more than 8 bytes for the header, and 6 bytes per value inserted
       let mut p = Page::new(0);
       assert!(p.get_header_size() <= FIXED_HEADER_SIZE);
       let byte_size = 10;
       let bytes = get_random_byte_vec(byte_size);
       // how many vals can we hold with 8 bytes
       let num_vals: usize = (((PAGE_SIZE - FIXED_HEADER_SIZE) as f64
           / (byte_size + HEADER_PER_VAL_SIZE) as f64)
           .floor()) as usize;
       if PAGE_SIZE == 4096 && FIXED_HEADER_SIZE == 8 && HEADER_PER_VAL_SIZE == 6 {
           assert_eq!(255, num_vals);
       }
       for _ in 0..num_vals {
           p.add_value(&bytes);
       }
       assert!(p.get_header_size() <= FIXED_HEADER_SIZE + (num_vals * HEADER_PER_VAL_SIZE));
       assert!(
           p.get_largest_free_contiguous_space()
               >= PAGE_SIZE
                   - (byte_size * num_vals)
                   - FIXED_HEADER_SIZE
                   - (num_vals * HEADER_PER_VAL_SIZE)
       );
   }
 
   #[test]
   fn hs_page_no_space() {
       init();
       let mut p = Page::new(0);
       let size = PAGE_SIZE / 4;
       let bytes = get_random_byte_vec(size);
       assert_eq!(Some(0), p.add_value(&bytes));
       assert_eq!(
           PAGE_SIZE - p.get_header_size() - size,
           p.get_largest_free_contiguous_space()
       );
       assert_eq!(Some(1), p.add_value(&bytes));
       assert_eq!(
           PAGE_SIZE - p.get_header_size() - size * 2,
           p.get_largest_free_contiguous_space()
       );
       assert_eq!(Some(2), p.add_value(&bytes));
       assert_eq!(
           PAGE_SIZE - p.get_header_size() - size * 3,
           p.get_largest_free_contiguous_space()
       );
       //Should reject here
       assert_eq!(None, p.add_value(&bytes));
       assert_eq!(
           PAGE_SIZE - p.get_header_size() - size * 3,
           p.get_largest_free_contiguous_space()
       );
       // Take small amount of data
       let small_bytes = get_random_byte_vec(size / 4);
       assert_eq!(Some(3), p.add_value(&small_bytes));
       assert_eq!(
           PAGE_SIZE - p.get_header_size() - size * 3 - small_bytes.len(),
           p.get_largest_free_contiguous_space()
       );
   }
 
   #[test]
   fn hs_page_simple_delete() {
       init();
       let mut p = Page::new(0);
       let tuple = int_vec_to_tuple(vec![0, 1, 2]);
       let tuple_bytes = serde_cbor::to_vec(&tuple).unwrap();
       assert_eq!(Some(0), p.add_value(&tuple_bytes));
       let check_bytes = p.get_value(0).unwrap();
       println!("fail");
       let check_tuple: Tuple = serde_cbor::from_slice(&check_bytes).unwrap();
       assert_eq!(tuple_bytes, check_bytes);
       assert_eq!(tuple, check_tuple);
       println!("pass one");
 
       let tuple2 = int_vec_to_tuple(vec![3, 3, 3]);
       let tuple_bytes2 = serde_cbor::to_vec(&tuple2).unwrap();
       assert_eq!(Some(1), p.add_value(&tuple_bytes2));
       let check_bytes2 = p.get_value(1).unwrap();
       println!("fail");
       let check_tuple2: Tuple = serde_cbor::from_slice(&check_bytes2).unwrap();
       assert_eq!(tuple_bytes2, check_bytes2);
       assert_eq!(tuple2, check_tuple2);
       println!("pass 2");
 
       //Delete slot 0
       println!("before delete");
       assert_eq!(Some(()), p.delete_value(0));
       println!("after delete");
       //Recheck slot 1
       let check_bytes2 = p.get_value(1).unwrap();
       let check_tuple2: Tuple = serde_cbor::from_slice(&check_bytes2).unwrap();
       assert_eq!(tuple_bytes2, check_bytes2);
       assert_eq!(tuple2, check_tuple2);
 
       //Verify slot 0 is gone
       println!("first none");
       assert_eq!(None, p.get_value(0));
       println!("second none");
       //Check that invalid slot gets None
       assert_eq!(None, p.get_value(2));
 
       //Delete slot 1
       assert_eq!(Some(()), p.delete_value(1));
 
       println!("third none");
       //Verify slot 0 is gone
       assert_eq!(None, p.get_value(1));
       println!("end");
   }
 
   #[test]
   fn hs_page_get_first_free_space() {
       init();
       let mut p = Page::new(0);
 
       let _b1 = get_random_byte_vec(100);
       let _b2 = get_random_byte_vec(50);
   }
 
   #[test]
   fn hs_page_delete_insert() {
       init();
       let mut p = Page::new(0);
       let tuple_bytes = get_random_byte_vec(20);
       let tuple_bytes2 = get_random_byte_vec(20);
       let tuple_bytes3 = get_random_byte_vec(20);
       let tuple_bytes4 = get_random_byte_vec(20);
       let tuple_bytes_big = get_random_byte_vec(40);
       let tuple_bytes_small1 = get_random_byte_vec(5);
       let tuple_bytes_small2 = get_random_byte_vec(5);
 
       //Add 3 values
       assert_eq!(Some(0), p.add_value(&tuple_bytes));
       let check_bytes = p.get_value(0).unwrap();
       assert_eq!(tuple_bytes, check_bytes);
       assert_eq!(Some(1), p.add_value(&tuple_bytes2));
 
       let check_bytes = p.get_value(1).unwrap();
       assert_eq!(tuple_bytes2, check_bytes);
       assert_eq!(Some(2), p.add_value(&tuple_bytes3));
 
       let check_bytes = p.get_value(2).unwrap();
       assert_eq!(tuple_bytes3, check_bytes);
       println!("PASS ADDING THREE VALUES");
 
       //Delete slot 1
       assert_eq!(Some(()), p.delete_value(1));
      
       //Verify slot 1 is gone
       assert_eq!(None, p.get_value(1));
 
       let check_bytes = p.get_value(0).unwrap();
       assert_eq!(tuple_bytes, check_bytes);
       let check_bytes = p.get_value(2).unwrap();
       assert_eq!(tuple_bytes3, check_bytes);
       println!("PASS DELETED SLOT 1 AND GET VALUES");
 
       //Insert same bytes, should go to slot 1
       assert_eq!(Some(1), p.add_value(&tuple_bytes4));
       println!("PASS FIRST OVERWRITE");
 
       let check_bytes = p.get_value(1).unwrap();
       assert_eq!(tuple_bytes4, check_bytes);
 
       //Delete 0
       assert_eq!(Some(()), p.delete_value(0));
       println!("PASS FIRST DELETE");
       println!("marie said pee pee poo poo! BING BONG");
 
       //Insert big, should go to slot 0 with space later in free block
       assert_eq!(Some(0), p.add_value(&tuple_bytes_big));
       println!("PASS second OVERWRITE");
 
       //Insert small, should go to 3
       assert_eq!(Some(3), p.add_value(&tuple_bytes_small1));
       println!("PASS SECOND LAST ADD VALUE");
       //Insert small, should go to new
       assert_eq!(Some(4), p.add_value(&tuple_bytes_small2));
       println!("PASS LAST ADD VALUE");
   }
 
   #[test]
   fn hs_page_size() {
       init();
       let mut p = Page::new(2);
       let tuple = int_vec_to_tuple(vec![0, 1, 2]);
       let tuple_bytes = serde_cbor::to_vec(&tuple).unwrap();
       assert_eq!(Some(0), p.add_value(&tuple_bytes));
 
       let page_bytes = p.get_bytes();
       assert_eq!(PAGE_SIZE, page_bytes.len());
   }
 
   #[test]
   fn hs_page_simple_byte_serialize() {
       init();
       let mut p = Page::new(0);
       let tuple = int_vec_to_tuple(vec![0, 1, 2]);
       let tuple_bytes = serde_cbor::to_vec(&tuple).unwrap();
       assert_eq!(Some(0), p.add_value(&tuple_bytes));
       let tuple2 = int_vec_to_tuple(vec![3, 3, 3]);
       let tuple_bytes2 = serde_cbor::to_vec(&tuple2).unwrap();
       println!("PASS ADD VALUE 2");
       assert_eq!(Some(1), p.add_value(&tuple_bytes2));
       println!("PASS START SERIALIZE");
       //Get bytes and create from bytes
       let bytes = p.get_bytes();
       println!("PASS GET BYTES");
       let mut p2 = Page::from_bytes(&bytes);
       println!("PASS FROM BYTES");
       assert_eq!(0, p2.get_page_id());
 
       //Check reads
       println!("START READS");
       let check_bytes2 = p2.get_value(1).unwrap();
       println!("PASS GET VALUE 1");
       let check_tuple2: Tuple = serde_cbor::from_slice(&check_bytes2).unwrap();
       assert_eq!(tuple_bytes2, check_bytes2);
       println!("In between");
       assert_eq!(tuple2, check_tuple2);
       println!("BEFORE GET VALUE 2");
       let check_bytes = p2.get_value(0).unwrap();
       println!("PASS GET VALUE 2");
       let check_tuple: Tuple = serde_cbor::from_slice(&check_bytes).unwrap();
       assert_eq!(tuple_bytes, check_bytes);
       assert_eq!(tuple, check_tuple);
 
       //Add a new tuple to the new page
       let tuple3 = int_vec_to_tuple(vec![4, 3, 2]);
       let tuple_bytes3 = tuple3.get_bytes();
       assert_eq!(Some(2), p2.add_value(&tuple_bytes3));
       assert_eq!(tuple_bytes3, p2.get_value(2).unwrap());
       assert_eq!(tuple_bytes2, p2.get_value(1).unwrap());
       assert_eq!(tuple_bytes, p2.get_value(0).unwrap());
   }
 
   #[test]
   fn hs_page_iter() {
       init();
       let mut p = Page::new(0);
       let tuple = int_vec_to_tuple(vec![0, 0, 1]);
       let tuple_bytes = serde_cbor::to_vec(&tuple).unwrap();
       assert_eq!(Some(0), p.add_value(&tuple_bytes));
 
       let tuple2 = int_vec_to_tuple(vec![0, 0, 2]);
       let tuple_bytes2 = serde_cbor::to_vec(&tuple2).unwrap();
       assert_eq!(Some(1), p.add_value(&tuple_bytes2));
 
       let tuple3 = int_vec_to_tuple(vec![0, 0, 3]);
       let tuple_bytes3 = serde_cbor::to_vec(&tuple3).unwrap();
       assert_eq!(Some(2), p.add_value(&tuple_bytes3));
 
       let tuple4 = int_vec_to_tuple(vec![0, 0, 4]);
       let tuple_bytes4 = serde_cbor::to_vec(&tuple4).unwrap();
       assert_eq!(Some(3), p.add_value(&tuple_bytes4));
 
       let tup_vec = vec![
           tuple_bytes.clone(),
           tuple_bytes2.clone(),
           tuple_bytes3.clone(),
           tuple_bytes4.clone(),
       ];
       let page_bytes = p.get_bytes();
 
       // Test iteration 1
       let mut iter = p.into_iter();
       assert_eq!(Some(tuple_bytes.clone()), iter.next());
       assert_eq!(Some(tuple_bytes2.clone()), iter.next());
       assert_eq!(Some(tuple_bytes3.clone()), iter.next());
       assert_eq!(Some(tuple_bytes4.clone()), iter.next());
       assert_eq!(None, iter.next());
 
       //Check another way
       let p = Page::from_bytes(&page_bytes);
       assert_eq!(Some(tuple_bytes.clone()), p.get_value(0));
 
       for (i, x) in p.into_iter().enumerate() {
           assert_eq!(tup_vec[i], x);
       }
 
       let p = Page::from_bytes(&page_bytes);
       let mut count = 0;
       for _ in p {
           count += 1;
       }
       assert_eq!(count, 4);
 
       //Add a value and check
       let mut p = Page::from_bytes(&page_bytes);
       assert_eq!(Some(4), p.add_value(&tuple_bytes));
       //get the updated bytes
       let page_bytes = p.get_bytes();
       count = 0;
       for _ in p {
           count += 1;
       }
       assert_eq!(count, 5);
 
       //Delete
       let mut p = Page::from_bytes(&page_bytes);
       p.delete_value(2);
       let mut iter = p.into_iter();
       assert_eq!(Some(tuple_bytes.clone()), iter.next());
       assert_eq!(Some(tuple_bytes2.clone()), iter.next());
       assert_eq!(Some(tuple_bytes4.clone()), iter.next());
       assert_eq!(Some(tuple_bytes.clone()), iter.next());
       assert_eq!(None, iter.next());
   }
}

