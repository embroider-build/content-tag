import { it, expect } from "vitest";
import { Preprocessor } from "content-tag";

const p = new Preprocessor();

it("works for a basic example", function () {
  let output = p.process("<template>Hi</template>");

  expect(output)
    .toMatchInlineSnapshot(`import { template } from "@ember/template-compiler";
    export default template(\`Hi\`, {
        eval () {
            return eval(arguments[0]);
        }
    });`);
});
