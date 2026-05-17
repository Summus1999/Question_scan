/**
 * Vitest 测试环境初始化
 *
 * 职责：导入 jest-dom 扩展匹配器，让测试可以使用 toBeInTheDocument() 等 DOM 断言。
 * 此文件在 vitest.config.ts 中通过 setupFiles 配置加载。
 */
import '@testing-library/jest-dom/vitest';
