let x = class {
  static {
    template(`Hello`, {
      component: this,
      eval() {
        return eval(arguments[0]);
      },
    });
  }
};