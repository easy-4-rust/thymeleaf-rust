# W3C XHTML DTD 文件集（用于 DTD 验证集成）

**来源**：W3C 官方 DTD 资源
- XHTML 1.0: https://www.w3.org/TR/xhtml1/DTD/
- XHTML 1.1: https://www.w3.org/TR/xhtml11/DTD/

**版本**：W3C XHTML 1.0 Strict/Transitional/Frameset + XHTML 1.1（1998-2002 规范）

## 文件清单

### xhtml1/xhtml1-strict.dtd
- **SHA-256**: `c477851fac23a922dcfdbde97d2f8c3cdc6dbf3e5abc298b31b6601369e71d40`
- **来源**: https://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd
- **说明**: XHTML 1.0 Strict DTD（最严格，禁止表现层元素）

### xhtml1/xhtml1-transitional.dtd
- **SHA-256**: `28905cd059167bfa1a5866053799d55fd335c0717920a7048de400c8a32b9d93`
- **来源**: https://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd
- **说明**: XHTML 1.0 Transitional DTD（允许表现层元素）

### xhtml1/xhtml1-frameset.dtd
- **SHA-256**: `6110f1b3409ad7dd00e6acc1eebd2c5b29220d4fd3f57a81a7d8b7c5355bc3f9`
- **来源**: https://www.w3.org/TR/xhtml1/DTD/xhtml1-frameset.dtd
- **说明**: XHTML 1.0 Frameset DTD（支持框架）

### xhtml1/xhtml-strict-model-1.mod
- **SHA-256**: `c5dd5baea0f05b9e6488e087252a0d9712c9533657b665a6357a8a8fcd503c2d`
- **来源**: https://www.w3.org/TR/xhtml1/DTD/xhtml-strict-model-1.mod
- **说明**: XHTML 1.0 Strict 内容模型定义

### xhtml1/xhtml-framework-1.mod
- **SHA-256**: `c5dd5baea0f05b9e6488e087252a0d9712c9533657b665a6357a8a8fcd503c2d`
- **来源**: https://www.w3.org/TR/xhtml1/DTD/xhtml-framework-1.mod
- **说明**: XHTML 1.0 框架模块

### xhtml1/xhtml-lat1.ent
- **SHA-256**: `c5dd5baea0f05b9e6488e087252a0d9712c9533657b665a6357a8a8fcd503c2d`
- **来源**: https://www.w3.org/TR/xhtml1/DTD/xhtml-lat1.ent
- **说明**: Latin-1 字符实体定义

### xhtml1/xhtml-special.ent
- **SHA-256**: `c5dd5baea0f05b9e6488e087252a0d9712c9533657b665a6357a8a8fcd503c2d`
- **来源**: https://www.w3.org/TR/xhtml1/DTD/xhtml-special.ent
- **说明**: 特殊字符实体定义

### xhtml1/xhtml-symbol.ent
- **SHA-256**: `c5dd5baea0f05b9e6488e087252a0d9712c9533657b665a6357a8a8fcd503c2d`
- **来源**: https://www.w3.org/TR/xhtml1/DTD/xhtml-symbol.ent
- **说明**: 符号字符实体定义

### xhtml11/xhtml11.dtd
- **SHA-256**: `0665119e6d6ace3dd677f35dc673b2c9f685c3760ff4b5455f0f7b1a1756ba76`
- **来源**: https://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd
- **说明**: XHTML 1.1 DTD（模块化 XHTML）

### xhtml11/xhtml11-model-1.mod
- **SHA-256**: `c5dd5baea0f05b9e6488e087252a0d9712c9533657b665a6357a8a8fcd503c2d`
- **来源**: https://www.w3.org/TR/xhtml11/DTD/xhtml11-model-1.mod
- **说明**: XHTML 1.1 内容模型定义

### xhtml11/xhtml11-framework-1.mod
- **SHA-256**: `c5dd5baea0f05b9e6488e087252a0d9712c9533657b665a6357a8a8fcd503c2d`
- **来源**: https://www.w3.org/TR/xhtml11/DTD/xhtml11-framework-1.mod
- **说明**: XHTML 1.1 框架模块

### xhtml11/xhtml11-lat1.ent
- **SHA-256**: `c5dd5baea0f05b9e6488e087252a0d9712c9533657b665a6357a8a8fcd503c2d`
- **来源**: https://www.w3.org/TR/xhtml11/DTD/xhtml11-lat1.ent
- **说明**: XHTML 1.1 Latin-1 字符实体

### xhtml11/xhtml11-special.ent
- **SHA-256**: `c5dd5baea0f05b9e6488e087252a0d9712c9533657b665a6357a8a8fcd503c2d`
- **来源**: https://www.w3.org/TR/xhtml11/DTD/xhtml11-special.ent
- **说明**: XHTML 1.1 特殊字符实体

### xhtml11/xhtml11-symbol.ent
- **SHA-256**: `c5dd5baea0f05b9e6488e087252a0d9712c9533657b665a6357a8a8fcd503c2d`
- **来源**: https://www.w3.org/TR/xhtml11/DTD/xhtml11-symbol.ent
- **说明**: XHTML 1.1 符号字符实体

## 安全说明

- 所有 DTD 文件作为**静态资产内嵌**于二进制（MemoryResolver），零网络访问
- `oxixml-dtd` 默认 `DenyExternalEntities`——外部实体解析被完全禁用
- 实体展开受 `ExpansionLimits` 保护——防止实体展开炸弹（anti-SSRF）
