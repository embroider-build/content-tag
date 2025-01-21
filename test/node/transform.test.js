import chai from "chai";
import { codeEquality } from "code-equality-assertions/chai";
import { transform } from "content-tag/utils";

chai.use(codeEquality);

const { expect } = chai;

describe(`transform`, () => {
  it('works', () => {
    const gjs = [
      "test('it renders', async (assert) => {",
      '  await render(<template>',
      '  <div class="parent">',
      '    <div class="child"></div>',
      '  </div>',
      '  </template>);',
      '});',
    ].join('\n');

    let result = transform(gjs, (hbs) => 'replaced!');

    expect(result).to.deep.equal([
      "test('it renders', async (assert) => {",
      '  await render(<template>replaced!</template>);',
      '});',
    ].join('\n'));
  });

  it('works on multiple', () => {
    const gjs = [
      "test('it renders', async (assert) => {",
      '  await render(<template>',
      '  <div class="parent">',
      '    <div class="child"></div>',
      '  </div>',
      '  </template>);',
      '});',
    ].join('\n');

    let result = transform(gjs, (hbs) => 'replaced!');

    expect(result).to.deep.equal([
      "test('it renders', async (assert) => {",
      '  await render(<template>replaced!</template>);',
      '});',
    ].join('\n'));
  });
});
