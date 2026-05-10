import { describe, expect, it } from 'vitest';
import { APP_MESSAGES, getMessages } from './i18n';

describe('interface localization', () => {
  it('defaults the stage shell copy to Chinese', () => {
    const messages = getMessages('zhCn');

    expect(messages.header.title).toBe('桌面应用底座');
    expect(messages.settings.title).toBe('设置');
    expect(messages.fields.interfaceLanguage).toBe('界面语言');
    expect(messages.localeOptions.enUs).toBe('English');
  });

  it('keeps English copy available for users who switch language', () => {
    const messages = getMessages('enUs');

    expect(messages.header.title).toBe('Desktop shell');
    expect(messages.runtime.title).toBe('Runtime snapshot');
    expect(messages.outputSpeed.normal).toBe('Normal');
    expect(messages.appStatus.ready).toBe('Ready');
  });

  it('keeps both locales structurally aligned', () => {
    expect(Object.keys(APP_MESSAGES.zhCn)).toEqual(
      Object.keys(APP_MESSAGES.enUs),
    );
    expect(Object.keys(APP_MESSAGES.zhCn.outputSpeed)).toEqual(
      Object.keys(APP_MESSAGES.enUs.outputSpeed),
    );
    expect(Object.keys(APP_MESSAGES.zhCn.theme)).toEqual(
      Object.keys(APP_MESSAGES.enUs.theme),
    );
  });
});
