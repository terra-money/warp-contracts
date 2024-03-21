import task from "@terra-money/terrariums";

task(async ({ deployer }) => {
  deployer.buildContract("warp-controller");
  deployer.optimizeContract("warp-controller");
});
