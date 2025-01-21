/**
 * Transforms each template within a gjs or gts file
 *
 * @param {string} source the original source
 * @param {(innerContent: string): string} eachTemplate the function to run on each of the contents (omitting the outer `<template>` and `</template>` tags)
 * @return {string}
 */
export function transform(
  source: string,
  eachTemplate: (innerContent: string) => string): string;
