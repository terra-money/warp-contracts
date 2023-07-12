import task from "@terra-money/terrariums";

task(async ({ deployer, signer, refs }) => {
  //account
  // deployer.buildContract("warp-account");
  // deployer.optimizeContract("warp-account");
  //
  // const id = await deployer.storeCode("warp-account");
  // await new Promise((resolve) => setTimeout(resolve, 10000));
  //
  // await deployer.storeCode("warp-resolver");
  // await new Promise((resolve) => setTimeout(resolve, 10000));
  //
  // //controller
  // // deployer.buildContract("warp-controller");
  // // deployer.optimizeContract("warp-controller");
  //
  // await deployer.storeCode("warp-controller");
  // await new Promise((resolve) => setTimeout(resolve, 10000));
  // //
  // const instantiateControllerMsg = {
  //   warp_account_code_id: id,
  //   creation_fee: "5",
  //   cancellation_fee: "5",
  //   minimum_reward: "10000",
  //   t_max: "86400",
  //   t_min: "86400",
  //   a_max: "10000",
  //   a_min: "10000",
  //   q_max: "10",
  // };
  //
  // await deployer.instantiate("warp-controller", instantiateControllerMsg, {
  //   admin: signer.key.accAddress,
  // });
  // await new Promise((resolve) => setTimeout(resolve, 10000));
  //
  const instantiateResolverMsg = {
    owner: signer.key.accAddress,
    fee_collector: signer.key.accAddress,
    templates: [],
  }

  await deployer.instantiate("warp-resolver", instantiateResolverMsg, {
    admin: signer.key.accAddress,
  });
  await new Promise((resolve) => setTimeout(resolve, 10000));

  refs.saveRefs();
});
