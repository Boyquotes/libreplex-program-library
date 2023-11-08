use std::cell::RefMut;

use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use anchor_lang::{AnchorDeserialize, AnchorSerialize};

#[derive(Clone, AnchorDeserialize, AnchorSerialize)]
pub enum SummaryExtension {
    None,
}

#[derive(Clone, AnchorDeserialize, AnchorSerialize)]
pub enum MediaType {
    Image,
    Erc721
}


#[derive(Clone, AnchorDeserialize, AnchorSerialize)]
pub enum EncodingType {
    Base64,
}

#[account]
pub struct InscriptionRankPage {
    pub size: u32,
}



impl InscriptionRankPage {
    // discriminator + vector size
    pub const BASE_SIZE: usize = 8 + 4;

    pub fn add_inscription(
        &mut self,
        mut current_data: RefMut<&mut [u8]>,
        inscription: Pubkey
    ) -> Result<()> {
        let data_length_max = u32::from_le_bytes(current_data[8..12].try_into().unwrap()) as usize;
        println!("data length {}", data_length_max);
        let data_slice: &mut [u8] =
            &mut current_data[(12 + data_length_max * 32)..(12 + (data_length_max +1)* 32)];
        data_slice.copy_from_slice(inscription.key().as_ref());

        self.size += 1;

        Ok(())
    }

    pub fn get_inscriptions<'a>(current_data: &'a RefMut<'a, &mut [u8]>, start_pos: usize, end_pos: usize) -> impl Iterator<Item=Pubkey> + 'a {

        let effective_start_pos = std::cmp::min(12 + start_pos*32, current_data.len());
        let effective_end_pos = std::cmp::min(12 + end_pos*32, current_data.len());

        println!("start: {}, end: {}", effective_start_pos, effective_end_pos);
        let byte_slice = &current_data[(effective_start_pos)..(effective_end_pos)];
        byte_slice.chunks(32).map(|x| Pubkey::try_from_slice(x).unwrap())
    }
}

#[account]
pub struct InscriptionSummary {
    pub inscription_count_total: u64,
    pub inscription_count_immutables: u64,
    pub last_inscription: Pubkey,
    pub last_inscriber: Pubkey,
    pub last_inscription_create_time: i64,
    pub extension: SummaryExtension,
}

impl InscriptionSummary {
    pub const BASE_SIZE: usize = 8 + 8 + 8 + 32 + 32 + 8 + 2 + 2;
}

#[account]
pub struct InscriptionData {
    // no explicit fields. This contains the inscription data itself
}

#[account]
pub struct Inscription {
    // no option to keep data easier to write into
    // set to 11111111.... or whatever to make this inscription immutable
    pub authority: Pubkey, // 8

    // root is the thing that the Inscription inscribes
    // could also be called inscribee but that would
    // be weird
    pub root: Pubkey, // 8 + 32 = 40


    // media type - image, erc721, mov, html, etc
    pub media_type: MediaType,

    pub encoding_type: EncodingType,

    // pointer to inscription data object. This allows us to keep the data
    // struct free of prefixes etc
    pub inscription_data: Pubkey, 

    // rank 0 - unranked. ranks 1,2,3,4,5,6 represent the rank of this inscription in the order they are made immutable
    // only immutable inscriptions are ranked.

    pub order: u64, // 8 + 32 + 32 = 72
    pub size: u32,    // 8 + 32 + 32 + 8 = 80
                      // we do not mark the following field as being serialized at all. instead we
                      // write to it directly via append_data method
                      // pub data: Vec<u8>
    /* 
        Validation hash is used to ensure that any inscription
        uploaded is in sync. This is important as uploading a 
        large inscription typically takes multiple transactions
        and we want to know whether the content was written 
        correctly and in its entirety.

        Validation hash can be updated by the inscription authority.

        For immutable inscriptions, the inscription authority is an
        account that cannot sign. Hence it's important we check the
        validation hash before allowing for immutability. Therefore 
        an inscription can only be made immutable if the inscription
        content validates against the hash.

        Validation hash is optional in case no validation is required.

    */
    pub validation_hash: Option<String>
}





impl Inscription {
    pub const BASE_SIZE: usize = 8 + 32 + 32 + 2 + 2 + 32 + 8 + 4 + 1 + 1; // no need for vector padding as we write bytes directly onto the account

    pub fn write_data(
        &self,
        mut current_data: RefMut<&mut [u8]>,
        data_to_add: &Vec<u8>,
        start_pos: u32,
    ) -> Result<()> {
        
        if start_pos + data_to_add.len() as u32 > self.size {
            return Err(ErrorCode::MaxSizeExceeded.into());
        }

        let current_index = start_pos as usize;
        let data_slice: &mut [u8] =
            &mut current_data[current_index..current_index + data_to_add.len()];
        data_slice.copy_from_slice(data_to_add);

        Ok(())
    }
}

#[derive(Clone, AnchorDeserialize, AnchorSerialize)]
pub enum InscriptionEventType {
    Create,
    Update,
    Resize,
}



#[derive(Clone, AnchorDeserialize, AnchorSerialize)]
pub struct InscriptionEventData {
    pub authority: Pubkey, // 8
    pub root: Pubkey, // 8 + 32 = 40
    pub media_type: MediaType,
    pub encoding_type: EncodingType,
    pub inscription_data: Pubkey, 

    pub order: u64, // 8 + 32 + 32 = 72
    pub size: u32,    // 8 + 32 + 32 + 8 = 80
    pub validation_hash: Option<String>
}