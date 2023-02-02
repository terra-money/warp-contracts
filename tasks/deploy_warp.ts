import task from "terrariums";

task(async ({ deployer, signer, refs }) => {
  //account
  deployer.buildContract("warp-account");
  deployer.optimizeContract("warp-account");

  const id = await deployer.storeCode("warp-account");
  await new Promise((resolve) => setTimeout(resolve, 3000));

  //controller
  deployer.buildContract("warp-controller");
  deployer.optimizeContract("warp-controller");

  await deployer.storeCode("warp-controller");
  await new Promise((resolve) => setTimeout(resolve, 3000));
  //
  const instantiateMsg = {
    warp_account_code_id: "7335",
    creation_fee: "0",
    cancellation_fee: "0",
    minimum_reward: "1",
    template_fee: "0",
    t_max: "60",
    t_min: "60",
    a_max: "1",
    a_min: "1",
    q_max: "10",
  };

  await deployer.instantiate("warp-controller", instantiateMsg, {
    admin: signer.key.accAddress,
  });

  refs.saveRefs();
});
