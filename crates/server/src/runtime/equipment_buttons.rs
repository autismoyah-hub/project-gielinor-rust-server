// A mapped static worn component/current item remains authoritative.
pub(super) fn worn_item_claim_matches(
    button: &gielinor_protocol::buttons::Button,
    item: i32,
) -> bool {
    button.slot == -1 && (button.object == -1 || button.object == item)
}
