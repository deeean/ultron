import * as ultron from '../index';


async function main() {
  const target = await ultron.readImageData('./example/testdata.png');

  const source = ultron.takeScreenshot(0, 0, 500, 500);
  const position = await ultron.imageSearch(source, target, 8);
}

main().then(() => {
  console.log('done');
});