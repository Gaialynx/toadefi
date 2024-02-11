use alloy_sol_macro::sol;

// how to make these publically scoped?

sol! {
     struct StreamAuthentication {
        bytes32 sender;
        uint64 expiration;
    }

    struct Order {
        bytes32 sender;
        int128 priceX18;
        int128 amount;
        uint64 expiration;
        uint64 nonce;
    }
}
