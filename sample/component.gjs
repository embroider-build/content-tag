import { template as t } from '@ember/template-compiler';

console.log(template);

const Inner = <template>I am inner</template>

// here's a comment
export class Outer {
  <template><Inner /></template>
}