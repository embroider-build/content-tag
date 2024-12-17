import { describe, it, expect } from 'vitest';


describe('package.json#export aliases', () => {
  it(`'.' === './standalone' when importing`, async () => {
    let main = await import('content-tag');
    let standalone = await import('content-tag/standalone');

    expect(main.Preprocessor).to.equal(standalone.Preprocessor);
  });
});
