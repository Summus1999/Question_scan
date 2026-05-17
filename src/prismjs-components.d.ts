/**
 * Prism.js 语言组件类型声明
 *
 * 职责：为动态导入的 Prism 语言组件提供空类型声明。
 * 这些模块没有导出类型，它们通过副作用自行注册到 Prism 对象上。
 */
// Type declarations for Prism.js language components loaded dynamically.
// These modules have no exported types; they register themselves with Prism.

declare module 'prismjs/components/prism-clike' {}
declare module 'prismjs/components/prism-c' {}
declare module 'prismjs/components/prism-cpp' {}
declare module 'prismjs/components/prism-python' {}
declare module 'prismjs/components/prism-java' {}
declare module 'prismjs/components/prism-javascript' {}
declare module 'prismjs/components/prism-typescript' {}
declare module 'prismjs/components/prism-go' {}
declare module 'prismjs/components/prism-rust' {}
declare module 'prismjs/components/prism-markdown' {}
