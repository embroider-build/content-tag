import { it, expect } from "vitest";
import { Preprocessor } from "content-tag";

const p = new Preprocessor();

it("is a browser", function () {
  // not that we don't trust vitest...
  expect(navigator.userAgent).to.include("Chrome");
});

it("basic example", function () {
  let output = p.parse("<template>Hello!</template>");

  expect(output).to.eql([
    {
      type: "expression",
      tagName: "template",
      contents: "Hello!",
      range: {
        start: 0,
        end: 27,
      },
      contentRange: {
        start: 10,
        end: 16,
      },
      startRange: {
        end: 10,
        start: 0,
      },
      endRange: {
        start: 16,
        end: 27,
      },
    },
  ]);
});
