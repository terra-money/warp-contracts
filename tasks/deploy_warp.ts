import task from "terrariums";

task(async ({ deployer, signer, refs }) => {
  //account
  // deployer.buildContract("warp-account");
  // deployer.optimizeContract("warp-account");

  // const account_id = await deployer.storeCode("warp-account");
  // await new Promise((resolve) => setTimeout(resolve, 5000));

  //resolver
  // deployer.buildContract("warp-resolver");
  // deployer.optimizeContract("warp-resolver")
  // const resolver_id = await deployer.storeCode("warp-resolver");
  // await new Promise((resolve) => setTimeout(resolve, 5000));
  // const resolver_promise = await deployer.instantiate("warp-resolver", {});
  // const resolver_address = resolver_promise.address
  // await new Promise((resolve) => setTimeout(resolve, 5000));


  //controller
  // deployer.buildContract("warp-controller");
  // deployer.optimizeContract("warp-controller");

  await deployer.storeCode("warp-controller");
  await new Promise((resolve) => setTimeout(resolve, 5000));
  //
  const instantiateMsg = {
    warp_account_code_id: "8858",
    resolver: "terra1ze03cu77l3dn3el40thky5hwh3gt9wctr4pn4qlyf28ce98ax4as29nx2z",
    creation_fee: "5",
    cancellation_fee: "5",
    minimum_reward: "10000",
    template_fee: "10000000",
    t_max: "86400",
    t_min: "86400",
    a_max: "10000",
    a_min: "10000",
    q_max: "10",
  };

  await deployer.instantiate("warp-controller", instantiateMsg, {
    admin: signer.key.accAddress,
  });

  refs.saveRefs();
});
