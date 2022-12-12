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

  const instantiateMsg = {
    warp_account_code_id: id,
    creation_fee: "0",
    cancellation_fee: "0",
    minimum_reward: "0",
  };

  await deployer.instantiate("warp-controller", instantiateMsg, {
    admin: signer.key.accAddress,
  });

  refs.saveRefs();
});
