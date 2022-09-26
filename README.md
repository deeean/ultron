# ultron
> 🚧 Attention! This is a work in progress. The API is not stable and may change at any time.

This repository is an automation framework for node.js

When I was young, I liked to automate things using AutoHotKey but it's not actively maintained, so I decided to make a new automation framework.

## Example
```typescript
import * as ultron from 'ultron';

async function main() {
  const target1 = await ultron.readImageData('./target1.png');
    const target2 = await ultron.readImageData('./target2.png');
  
  const source = ultron.takeScreenshot(0, 0, 500, 500);
  
  const position1 = await ultron.imageSearch(source, target1, 8);
  if (position1) {
    console.log(`Found at ${position1.x}, ${position1.y}`);
  } else {
    console.log('Not found');
  }

  const position2 = await ultron.imageSearch(source, target2, 8);
  if (position2) {
    console.log(`Found at ${position2.x}, ${position2.y}`);
  } else {
    console.log('Not found');
  }
}

main().then(() => {
  console.log('Done');
});
```