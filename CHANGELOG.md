# Changelog

## [0.4.1](https://github.com/TimG9v/ai-for-developers-project-387/compare/v0.4.0...v0.4.1) (2026-10-09)


### Bug Fixes

* **backend:** тест working-hours не зависит от дня недели ([56ff7bd](https://github.com/TimG9v/ai-for-developers-project-387/commit/56ff7bd5168a401730c3d1870dc25989ba41767c))
* **backend:** тест working-hours не зависит от дня недели ([466550c](https://github.com/TimG9v/ai-for-developers-project-387/commit/466550c6f72b1f26f8287557e15a386f753bf385))

## [0.4.0](https://github.com/TimG9v/ai-for-developers-project-387/compare/v0.3.2...v0.4.0) (2026-10-08)


### Features

* **frontend:** форма рабочих часов в админке и подпись часового пояса для гостя ([f17c687](https://github.com/TimG9v/ai-for-developers-project-387/commit/f17c687e1b63cfc347cb5798dd03d1aa3aa9eea0))
* рабочие окна расписания вместо ручных слотов по одному ([f8174f7](https://github.com/TimG9v/ai-for-developers-project-387/commit/f8174f72171402ccae2245daec7a71ab16e512c1)), closes [#9](https://github.com/TimG9v/ai-for-developers-project-387/issues/9)


### Bug Fixes

* **frontend:** обновить next до 16.4.0 — high-адвайзори GHSA ([3c69569](https://github.com/TimG9v/ai-for-developers-project-387/commit/3c695692d7dc7c5b7a03d898720d837f1fea4924))
* **frontend:** обновить next до 16.4.0 — high-адвайзори GHSA ([ca68bbe](https://github.com/TimG9v/ai-for-developers-project-387/commit/ca68bbe46a9357c5d7caaf35305232e5c3c9fc53))
* дефолт зоны формы из браузера и выравнивание окна горизонта в тесте ([2a24599](https://github.com/TimG9v/ai-for-developers-project-387/commit/2a245990d2ef9f6c53d9274309632609a01777a8)), closes [#9](https://github.com/TimG9v/ai-for-developers-project-387/issues/9)

## [0.3.2](https://github.com/TimG9v/ai-for-developers-project-387/compare/v0.3.1...v0.3.2) (2026-10-08)


### Bug Fixes

* дубли типов и слотов отклоняются 409 ([#19](https://github.com/TimG9v/ai-for-developers-project-387/issues/19)) ([66c8316](https://github.com/TimG9v/ai-for-developers-project-387/commit/66c8316220bfe06e51f11611db3852b94fb42558))

## [0.3.1](https://github.com/TimG9v/ai-for-developers-project-387/compare/v0.3.0...v0.3.1) (2026-10-08)


### Bug Fixes

* **ci:** no-op push в opencode-review — ревью read-only ([2ff1f8a](https://github.com/TimG9v/ai-for-developers-project-387/commit/2ff1f8a48c5eecb38c9e2d6c96ae6ff1f6673b67))
* **ci:** не запускать opencode-воркфлоу на событиях авторов-ботов ([fcc5b9e](https://github.com/TimG9v/ai-for-developers-project-387/commit/fcc5b9eb77017c6f6a07b1d94461052038d93bcc))
* **ci:** не запускать opencode-воркфлоу на событиях авторов-ботов ([8869cf5](https://github.com/TimG9v/ai-for-developers-project-387/commit/8869cf589a72b9fb6cec555f8018fa06b3442d02))
* **ci:** разрешить shallow-push в no-op bare для opencode-review ([322e012](https://github.com/TimG9v/ai-for-developers-project-387/commit/322e01273550c99afc1205f7d52177fa15dc2966))

## [0.3.0](https://github.com/TimG9v/ai-for-developers-project-387/compare/v0.2.0...v0.3.0) (2026-10-07)


### Features

* **backend:** демо-данные in-memory при старте бинарника ([a0ac267](https://github.com/TimG9v/ai-for-developers-project-387/commit/a0ac2674c4315c1edc7dcb356bce0519b46e3b7e))


### Bug Fixes

* **backend:** атомарный 409 на пересечение интервалов записей ([c132971](https://github.com/TimG9v/ai-for-developers-project-387/commit/c13297105a239fa1ff63af1b56e362d57c7911d5))
* **backend:** отклонять слоты вне 30-минутной сетки ([22e32bb](https://github.com/TimG9v/ai-for-developers-project-387/commit/22e32bb3f069963e769716d2e27efcd62e0d16eb))
* **backend:** скрывать из календаря слоты, пересекающие занятые интервалы ([92a1fe9](https://github.com/TimG9v/ai-for-developers-project-387/commit/92a1fe91d2b93c96c848cf26eaf8361af85f8d86))
* **frontend:** 30-минутная сетка в поле времени слота ([5c88362](https://github.com/TimG9v/ai-for-developers-project-387/commit/5c88362dfd4c493acbee8bb160ff53ea98738fe6))
* **frontend:** подсказывать причину отказа при публикации слота ([79d31cc](https://github.com/TimG9v/ai-for-developers-project-387/commit/79d31cc5a611f0a11407d17195b670f0b9fb1022))

## [0.2.0](https://github.com/TimG9v/ai-for-developers-project-387/compare/v0.1.0...v0.2.0) (2026-10-06)


### Features

* **admin:** подтвердить отмену записи из админки ([d32c383](https://github.com/TimG9v/ai-for-developers-project-387/commit/d32c3833080704bead98ef03f0ae2d4fe0134bbd))
* **api:** добавить отмену и перенос записи в контракт ([0f9ff0d](https://github.com/TimG9v/ai-for-developers-project-387/commit/0f9ff0d3401b6667882ee4921ed6b4c9c2796091))
* **backend:** роуты отмены и переноса записи ([aeeccf5](https://github.com/TimG9v/ai-for-developers-project-387/commit/aeeccf5df33f6d8f1357a16f91a0a30434c54427))
* **frontend:** управление записью по ссылке и отмена из админки ([5fb2d28](https://github.com/TimG9v/ai-for-developers-project-387/commit/5fb2d28a7ba3a7aaaf5a823a99e6e32d375f2feb))


### Bug Fixes

* **deps:** поднять source-map-js до 1.2.2, вернуть package-lock к состоянию main ([ab9009d](https://github.com/TimG9v/ai-for-developers-project-387/commit/ab9009da450a247e4c93226b62024f8f0f3a8f6a))
* **frontend:** обработать сетевую ошибку загрузки записи ([4638f87](https://github.com/TimG9v/ai-for-developers-project-387/commit/4638f873291bed7d22a97b92637972bcc53a6953))
