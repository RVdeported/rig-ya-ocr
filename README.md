## RIG-YA-OCR

Bad implementation of Yandex OCR [Rig](https://github.com/0xPlaygrounds/rig) plugin. Allow to use both Api-Key and IAM token auth. Please see more on yandex [Auth system](https://yandex.cloud/ru/docs/iam/). Note that the temporary tokens require 'yc' utility configured.

Key features:
- Supports both yandex auth types
- Supports dynamic client builder (but you still need to Register model, see the example)

Yandex OCR itself supports any image and PDF file with some restrictions as highlighted [here](https://yandex.cloud/ru/docs/vision/). Note that only One pdf page can be processed by the service.

Refer to the main file for usage examples.

In future:
- Support of all Yandex models
- Ready-to use patch (and pull-request) to fully embed the plugin into Rig. 

