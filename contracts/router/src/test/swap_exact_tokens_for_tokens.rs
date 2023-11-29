use crate::test::{SoroswapRouterTest, create_token_contract};
use crate::test::add_liquidity::add_liquidity;

use soroban_sdk::{
    Address,
    // testutils::{
        
    //     Ledger},
    vec, Vec};

#[test]
#[should_panic(expected = "SoroswapRouter: not yet initialized")] 
fn swap_exact_tokens_for_tokens_not_initialized() {
    let test = SoroswapRouterTest::setup();
    test.env.budget().reset_unlimited();
    let path: Vec<Address> = Vec::new(&test.env);
    test.contract.swap_exact_tokens_for_tokens(
        &0, //amount_in
        &0,  // amount_out_min
        &path, // path
        &test.user, // to
        &0); // deadline
}

#[test]
#[should_panic(expected = "SoroswapRouter: negative amount is not allowed: -1")]
fn swap_exact_tokens_for_tokens_amount_in_negative() {
    let test = SoroswapRouterTest::setup();
    test.env.budget().reset_unlimited();

    test.contract.initialize(&test.factory.address);
    let path: Vec<Address> = Vec::new(&test.env);
    test.contract.swap_exact_tokens_for_tokens(
        &-1, //amount_in
        &0,  // amount_out_min
        &path, // path
        &test.user, // to
        &0); // deadline
}


#[test]
#[should_panic(expected = "SoroswapRouter: negative amount is not allowed: -1")]
fn swap_exact_tokens_for_tokens_amount_out_min_negative() {
    let test = SoroswapRouterTest::setup();
    test.env.budget().reset_unlimited();

    test.contract.initialize(&test.factory.address);
    let path: Vec<Address> = Vec::new(&test.env);
    test.contract.swap_exact_tokens_for_tokens(
        &0, //amount_in
        &-1,  // amount_out_min
        &path, // path
        &test.user, // to
        &0); // deadline
}


#[test]
#[should_panic(expected = "SoroswapRouter: expired")]
fn swap_exact_tokens_for_tokens_expired() {
    let test = SoroswapRouterTest::setup();
    test.contract.initialize(&test.factory.address);
    let path: Vec<Address> = Vec::new(&test.env);
    test.contract.swap_exact_tokens_for_tokens(
        &0, //amount_in
        &0,  // amount_out_min
        &path, // path
        &test.user, // to
        &0); // deadline
}


#[test]
#[should_panic(expected = "SoroswapLibrary: invalid path")]
fn swap_exact_tokens_for_tokens_invalid_path() {
    let test = SoroswapRouterTest::setup();
    test.contract.initialize(&test.factory.address);
    let deadline: u64 = test.env.ledger().timestamp() + 1000;    
    let path: Vec<Address> =  vec![&test.env, test.token_0.address.clone()];

    test.contract.swap_exact_tokens_for_tokens(
        &0, //amount_in
        &0,  // amount_out_min
        &path, // path
        &test.user, // to
        &deadline); // deadline
}


#[test]
// Panics because LP does not exist; here panics with a Error(Storage, MissingValue)
// We should implement a pair_address.exist() without needing to call the Factory
#[should_panic]
fn swap_exact_tokens_for_tokens_pair_does_not_exist() {
    let test = SoroswapRouterTest::setup();
    test.contract.initialize(&test.factory.address);
    let deadline: u64 = test.env.ledger().timestamp() + 1000;  

    let mut path: Vec<Address> = Vec::new(&test.env);
    path.push_back(test.token_0.address.clone());
    path.push_back(test.token_1.address.clone());

    test.contract.swap_exact_tokens_for_tokens(
        &0, //amount_in
        &0,  // amount_out_min
        &path, // path
        &test.user, // to
        &deadline); // deadline
}


#[test]
#[should_panic(expected = "SoroswapLibrary: insufficient input amount")]
fn swap_exact_tokens_for_tokens_insufficient_input_amount() {
    let test = SoroswapRouterTest::setup();
    test.contract.initialize(&test.factory.address);
    let deadline: u64 = test.env.ledger().timestamp() + 1000;  

    let mut path: Vec<Address> = Vec::new(&test.env);
    path.push_back(test.token_0.address.clone());
    path.push_back(test.token_1.address.clone());

    let amount_0: i128 = 1_000_000_000_000_000_000;
    let amount_1: i128 = 4_000_000_000_000_000_000;

    add_liquidity(&test, &amount_0, &amount_1);

    test.env.budget().reset_unlimited();
    test.contract.swap_exact_tokens_for_tokens(
        &0, //amount_in
        &0,  // amount_out_min
        &path, // path
        &test.user, // to
        &deadline); // deadline

}


#[test]
fn swap_exact_tokens_for_tokens_enough_output_amount() {
    let test = SoroswapRouterTest::setup();
    test.contract.initialize(&test.factory.address);
    let deadline: u64 = test.env.ledger().timestamp() + 1000;  

    let mut path: Vec<Address> = Vec::new(&test.env);
    path.push_back(test.token_0.address.clone());
    path.push_back(test.token_1.address.clone());

    let amount_0: i128 = 1_000_000_000_000_000_000;
    let amount_1: i128 = 4_000_000_000_000_000_000;

    add_liquidity(&test, &amount_0, &amount_1);

    let amount_in = 1_000_000;

    //(1000000×997×4000000000000000000)÷(1000000000000000000×1000+997×1000000) = 3987999,9

    let expected_amount_out = 3987999;

    test.env.budget().reset_unlimited();
    test.contract.swap_exact_tokens_for_tokens(
        &amount_in, //amount_in
        &(expected_amount_out),  // amount_out_min
        &path, // path
        &test.user, // to
        &deadline); // deadline
}

#[test]
#[should_panic(expected = "SoroswapRouter: insufficient output amount")]
fn swap_exact_tokens_for_tokens_insufficient_output_amount() {
    let test = SoroswapRouterTest::setup();
    test.contract.initialize(&test.factory.address);
    let deadline: u64 = test.env.ledger().timestamp() + 1000;  

    let mut path: Vec<Address> = Vec::new(&test.env);
    path.push_back(test.token_0.address.clone());
    path.push_back(test.token_1.address.clone());

    let amount_0: i128 = 1_000_000_000_000_000_000;
    let amount_1: i128 = 4_000_000_000_000_000_000;

    add_liquidity(&test, &amount_0, &amount_1);

    let amount_in = 1_000_000;

    //(1000000×997×4000000000000000000)÷(1000000000000000000×1000+997×1000000) = 3987999,9

    let expected_amount_out = 3987999;

    test.env.budget().reset_unlimited();
    test.contract.swap_exact_tokens_for_tokens(
        &amount_in, //amount_in
        &(expected_amount_out+1),  // amount_out_min
        &path, // path
        &test.user, // to
        &deadline); // deadline
}

// #[test]
// fn swap_tokens_for_exact_tokens_amount_in_should() {
//     let test = SoroswapRouterTest::setup();
//     test.env.budget().reset_unlimited();
//     test.contract.initialize(&test.factory.address);
//     let deadline: u64 = test.env.ledger().timestamp() + 1000;  

//     let mut path: Vec<Address> = Vec::new(&test.env);
//     path.push_back(test.token_0.address.clone());
//     path.push_back(test.token_1.address.clone());

//     let amount_0: i128 = 1_000_000_000;
//     let amount_1: i128 = 4_000_000_000;

//     add_liquidity(&test, &amount_0, &amount_1);

//     let expected_amount_out = 5_000_000;
//     let amount_in_should = test.contract.router_get_amounts_in(&expected_amount_out, &path).get(0).unwrap();

//     let amounts = test.contract.swap_tokens_for_exact_tokens(
//         &expected_amount_out, //amount_out
//         &(amount_in_should),  // amount_in_max
//         &path, // path
//         &test.user, // to
//         &deadline); // deadline

//     assert_eq!(amounts.get(0).unwrap(), amount_in_should);
//     assert_eq!(amounts.get(1).unwrap(), expected_amount_out);

//     let original_balance: i128 = 10_000_000_000_000_000_000;
//     let expected_amount_0_in = 1255331;
//     assert_eq!(expected_amount_0_in, amount_in_should);
//     assert_eq!(test.token_0.balance(&test.user), original_balance - amount_0 - expected_amount_0_in);
//     assert_eq!(test.token_1.balance(&test.user), original_balance - amount_1 + expected_amount_out);

//     let pair_address = test.factory.get_pair(&test.token_0.address, &test.token_1.address);
//     assert_eq!(test.token_0.balance(&pair_address), amount_0 + expected_amount_0_in);
//     assert_eq!(test.token_1.balance(&pair_address), amount_1 - expected_amount_out);

// }


// #[test]
// fn swap_tokens_for_exact_tokens_more_amount_in_max() {
//     let test = SoroswapRouterTest::setup();
//     test.env.budget().reset_unlimited();
//     test.contract.initialize(&test.factory.address);

//     let amount_0: i128 = 1_000_000_000_000_000_000;
//     let amount_1: i128 = 4_000_000_000_000_000_000;

//     add_liquidity(&test, &amount_0, &amount_1);

//     let mut path: Vec<Address> = Vec::new(&test.env);
//     path.push_back(test.token_0.address.clone());
//     path.push_back(test.token_1.address.clone());

//     let expected_amount_out = 5_000_000;
//     // For a 1 swap, get_amounts_in returns [input, output]
//     let amount_in_should = test.contract.router_get_amounts_in(&expected_amount_out, &path).get(0).unwrap();

//     let deadline: u64 = test.env.ledger().timestamp() + 1000;  

//     test.contract.swap_tokens_for_exact_tokens(
//         &expected_amount_out, //amount_out
//         &(amount_in_should + 1_000_000_000_000_000_000),  // amount_in_max
//         &path, // path
//         &test.user, // to
//         &deadline); // deadline

// }



// #[test]
// fn swap_tokens_for_exact_tokens_2_hops() {
//     let test = SoroswapRouterTest::setup();
//     test.env.budget().reset_unlimited();
//     test.contract.initialize(&test.factory.address);
//     let deadline: u64 = test.env.ledger().timestamp() + 1000;  

//     let token_2 = create_token_contract(&test.env, &test.admin);

//     let amount_0: i128 = 1_000_000_000;
//     let amount_1: i128 = 4_000_000_000;

//     test.contract.add_liquidity(
//         &test.token_0.address, //     token_a: Address,
//         &test.token_1.address, //     token_b: Address,
//         &amount_0, //     amount_a_desired: i128,
//         &amount_1, //     amount_b_desired: i128,
//         &0, //     amount_a_min: i128,
//         &0 , //     amount_b_min: i128,
//         &test.user, //     to: Address,
//         &deadline//     deadline: u64,
//     );

//     let amount_2: i128 = 8_000_000_000;

//     test.contract.add_liquidity(
//         &test.token_1.address, //     token_a: Address,
//         &test.token_2.address, //     token_b: Address,
//         &amount_1, //     amount_a_desired: i128,
//         &amount_2, //     amount_b_desired: i128,
//         &0, //     amount_a_min: i128,
//         &0 , //     amount_b_min: i128,
//         &test.user, //     to: Address,
//         &deadline//     deadline: u64,
//     );
    
    
//     let mut path: Vec<Address> = Vec::new(&test.env);
//     path.push_back(test.token_0.address.clone());
//     path.push_back(test.token_1.address.clone());
//     path.push_back(test.token_2.address.clone());


//     let expected_amount_out = 5_000_000;
//     // First in = (4000000000×5000000×1000)÷((8000000000−5000000)×997) = 2509090
//     // Second in = (    )
//     let amount_in_should = test.contract.router_get_amounts_in(&expected_amount_out, &path).get(0).unwrap();

//     let amounts = test.contract.swap_tokens_for_exact_tokens(
//         &expected_amount_out, //amount_out
//         &(amount_in_should),  // amount_in_max
//         &path, // path
//         &test.user, // to
//         &deadline); // deadline

//     assert_eq!(amounts.get(0).unwrap(), amount_in_should);
//     assert_eq!(amounts.get(1).unwrap(), expected_amount_out);

//     let original_balance: i128 = 10_000_000_000_000_000_000;
//     let expected_amount_0_in = 1255331;
//     assert_eq!(expected_amount_0_in, amount_in_should);
//     assert_eq!(test.token_0.balance(&test.user), original_balance - amount_0 - expected_amount_0_in);
//     assert_eq!(test.token_1.balance(&test.user), original_balance - amount_1 + expected_amount_out);

//     let pair_address = test.factory.get_pair(&test.token_0.address, &test.token_1.address);
//     assert_eq!(test.token_0.balance(&pair_address), amount_0 + expected_amount_0_in);
//     assert_eq!(test.token_1.balance(&pair_address), amount_1 - expected_amount_out);

// }
