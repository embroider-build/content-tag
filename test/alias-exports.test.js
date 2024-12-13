import chai from 'chai';

const { expect } = chai;


describe('package.json#export aliases', () => {
  it(`'.' === './standalone' when importing`, async () => {
    let main = await import('content-tag');
    let standalone = await import('content-tag/standalone');

    expect(main.Preprocessor).to.equal(standalone.Preprocessor);
  });
});
