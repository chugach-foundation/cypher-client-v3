use {
    agnostic_orderbook::state::{
        critbit::{InnerNode, LeafNode, Slab, SlabHeader},
        event_queue::{EventQueueHeader, FillEvent},
        AccountTag,
    },
    anchor_lang::prelude::*,
    borsh::{BorshDeserialize, BorshSerialize},
    bytemuck::{Pod, Zeroable},
};

#[derive(
    Default, BorshDeserialize, BorshSerialize, Debug, Clone, Copy, Zeroable, Pod, PartialEq,
)]
#[repr(C)]
/// Information about a user involved in an orderbook matching event
pub struct CallBackInfo {
    #[allow(missing_docs)]
    pub user_account: Pubkey,
    #[allow(missing_docs)]
    pub fee_tier: u8,
    #[allow(missing_docs)]
    pub sub_account_idx: u8,
}

pub fn load_book_side<'a>(
    account_data: &'a mut [u8],
    expected_tag: AccountTag,
) -> Slab<'a, CallBackInfo> {
    let callback_info_len = std::mem::size_of::<CallBackInfo>();
    let leaf_size = LeafNode::LEN + callback_info_len;
    let capacity =
        (account_data.len() - SlabHeader::LEN - 8 - leaf_size) / (leaf_size + InnerNode::LEN);
    assert!(account_data[0] == expected_tag as u8);

    let (_, rem) = account_data.split_at_mut(8);
    let (header, rem) = rem.split_at_mut(SlabHeader::LEN);
    let (leaves, rem) = rem.split_at_mut((capacity + 1) * LeafNode::LEN);
    let (inner_nodes, callback_infos) = rem.split_at_mut(capacity * InnerNode::LEN);

    let header = bytemuck::from_bytes_mut::<SlabHeader>(header);

    Slab {
        header,
        leaf_nodes: bytemuck::cast_slice_mut::<_, LeafNode>(leaves),
        inner_nodes: bytemuck::cast_slice_mut::<_, InnerNode>(inner_nodes),
        callback_infos: bytemuck::cast_slice_mut::<_, CallBackInfo>(callback_infos),
    }
}

pub fn parse_aob_event_queue(
    account_data: &[u8],
) -> (&EventQueueHeader, &[FillEvent], &[CallBackInfo]) {
    let callback_info_len = std::mem::size_of::<CallBackInfo>();
    let capacity =
        (account_data.len() - 8 - EventQueueHeader::LEN) / (FillEvent::LEN + 2 * callback_info_len);

    let account_tag = *bytemuck::from_bytes::<u64>(&account_data[0..8]);
    assert!(account_tag == AccountTag::EventQueue as u64);

    let (header, remaining) = account_data[8..].split_at(EventQueueHeader::LEN);
    let header: &EventQueueHeader = bytemuck::from_bytes(header);

    let (events, callback_infos) = remaining.split_at(capacity * FillEvent::LEN);
    let events = bytemuck::cast_slice(events);

    let callback_infos = bytemuck::cast_slice(callback_infos);
    (header, events, callback_infos)
}
