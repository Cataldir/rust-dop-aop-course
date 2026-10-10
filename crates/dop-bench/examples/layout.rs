use std::mem::{align_of,offset_of,size_of};
#[repr(C)] struct Wire { flag:u8,timestamp:u64,value:f32 }
#[repr(C)] struct Reordered { timestamp:u64,value:f32,flag:u8 }
fn main() { println!("Wire size={} align={} timestamp_offset={}",size_of::<Wire>(),align_of::<Wire>(),offset_of!(Wire,timestamp));println!("Reordered size={} align={}",size_of::<Reordered>(),align_of::<Reordered>()); }