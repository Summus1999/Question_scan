/**
 * Question Scan 前端入口文件
 *
 * 职责：初始化 React 根节点，挂载 App 组件，启用 StrictMode。
 * 这是整个前端应用的起点，所有 UI 渲染从这里开始。
 */
import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './styles.css';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
