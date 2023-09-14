const Inner = <template>I am inner {{yield}}</template>

// here's a comment
export class Outer {
  <template>
    <Inner>
      Hello world
    </Inner>
  </template>
}