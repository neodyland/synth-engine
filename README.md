# License
This project is distributed under the MIT License.

Some files included in this project are derived from software distributed under the BSD 3-Clause License. The copyright notices and license texts for those components are preserved as required by their respective licenses.

Please note that both the MIT License and the BSD 3-Clause License require copyright notices and license terms to be retained when redistributing covered source code or substantial portions thereof. If you redistribute this project or derivative works, please ensure that all applicable license notices are preserved.

See the LICENSE and THIRD_PARTY_NOTICES files for details.

# 前提
HTS-Engine APIをインストールする必要があります。

# 実行方法
1. https://huggingface.co/neody/synth-engine-assets から `all.csv` 及び `symbol.csv` をダウンロードし、 `data/` 以下に配置します。
2. `./data/template.config.toml` を `./data/config.toml` にコピーします。
3. 適当な `.htsvoice` ファイルをダウンロードし(調べてください)、それにあわせて `./data/config.toml` を編集します。
4. `cargo run -r --bin synth-server` を実行します。

# API
## HTSモデル一覧
```sh
curl http://localhost:3000/v1/hts/models
```
## 合成
```sh
curl http://localhost:3000/v1/hts -X POST -H 'Content-Type: application/json' -d '{"text":"本日12/25はchristmasです！","model":"{適当なモデル名}"}' --output output.wav
```
## プリプロセッサ単体
```sh
curl http://localhost:3000/v1/preprocess -X POST -H 'Content-Type: application/json' -d '{"text":"本日12/25はchristmasです。6:30は早朝です。"}'
```

# その他ライセンス上の注意事項
https://huggingface.co/neody/synth-engine-assets に配置されている `all.csv` は、Wikipedia のダンプから取得した日英の記事タイトルを基に作成されています。

生成過程では、カテゴリページ、利用者ページ、テンプレートページなどの Wikipedia 固有の管理用ページを除外し、さらに各項目についてフィルタリングおよび加工を行っています。また、日本語表記については、既存のカタカナ表記の利用に加え、LLM（Qwen3.5-9B）による統計的推定を用いてカタカナ読みを生成したものが含まれます。

本データセットには Wikipedia の記事本文や説明文は含まれておらず、主として名称およびそれらを基に生成されたカタカナ表記から構成されています。そのため、本プロジェクトでは一般に Wikipedia の `cc-by-sa` ライセンスの継承対象には当たらないと考えています。

ただし、著作権およびデータベース権に関する判断は国や地域の法制度、ならびに個別の事実関係によって異なる場合があります。本記載は法的助言を構成するものではありません。
