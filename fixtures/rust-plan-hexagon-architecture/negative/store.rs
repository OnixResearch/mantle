fn leak(value: crunch_store::StoreHandle) {
    drop(value);
}
