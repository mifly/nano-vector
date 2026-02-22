import 'package:flutter/material.dart';
import 'package:nano_vector_app/main.dart';
import 'package:speech_to_text/speech_to_text.dart' as stt;

class IndexPage extends StatefulWidget {
  const IndexPage({super.key});

  @override
  State<IndexPage> createState() => _IndexPageState();
}

class _IndexPageState extends State<IndexPage> {
  final _textController = TextEditingController();
  bool _isSaving = false;

  final stt.SpeechToText _speechToText = stt.SpeechToText();
  bool _speechEnabled = false;
  bool _isListening = false;
  String _textBeforeListen = '';

  @override
  void initState() {
    super.initState();
    _initSpeech();
  }

  Future<void> _initSpeech() async {
    _speechEnabled = await _speechToText.initialize(
      onError: (val) => debugPrint('onError: $val'),
      onStatus: (val) {
        debugPrint('onStatus: $val');
        if (val == 'done' || val == 'notListening') {
          if (mounted) {
            setState(() {
              _isListening = false;
            });
          }
        }
      },
    );
    if (mounted) {
      setState(() {});
    }
  }

  Future<void> _startListening() async {
    // speech_to_text plugin handles permissions internally via initialize()
    if (!_speechEnabled) {
      // Try to re-initialize if not enabled
      _speechEnabled = await _speechToText.initialize(
        onError: (val) => debugPrint('onError: $val'),
        onStatus: (val) {
          debugPrint('onStatus: $val');
          if (val == 'done' || val == 'notListening') {
            if (mounted) {
              setState(() {
                _isListening = false;
              });
            }
          }
        },
      );
    }

    if (_speechEnabled) {
      _textBeforeListen = _textController.text;
      if (_textBeforeListen.isNotEmpty &&
          !_textBeforeListen.endsWith(' ') &&
          !_textBeforeListen.endsWith('\n')) {
        _textBeforeListen += ' ';
      }
      
      setState(() {
        _isListening = true;
      });
      
      await _speechToText.listen(
        localeId: 'zh_CN',
        onResult: (result) {
          if (mounted) {
            setState(() {
              _textController.text = _textBeforeListen + result.recognizedWords;
              // Keep cursor at the end
              _textController.selection = TextSelection.fromPosition(
                TextPosition(offset: _textController.text.length),
              );
            });
          }
        },
      );
    } else {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('设备不支持语音识别或初始化失败')),
        );
      }
    }
  }

  Future<void> _stopListening() async {
    await _speechToText.stop();
    if (mounted) {
      setState(() {
        _isListening = false;
      });
    }
  }

  void _toggleListening() {
    if (_isListening) {
      _stopListening();
    } else {
      _startListening();
    }
  }

  Future<void> _saveText() async {
    if (_isListening) {
      await _stopListening();
    }

    if (!mounted) return;

    final text = _textController.text.trim();
    if (text.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('请输入要索引的文本内容')),
      );
      return;
    }

    setState(() {
      _isSaving = true;
    });

    try {
      await appCore.indexText(text: text);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('保存成功！文本已被分片并存入向量库。')),
        );
        _textController.clear();
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('保存失败: $e')),
        );
      }
    } finally {
      if (mounted) {
        setState(() {
          _isSaving = false;
        });
      }
    }
  }

  @override
  void dispose() {
    _speechToText.cancel();
    _textController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(16.0),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text(
            '将文本分片后保存到关系型数据库 (SQLite) 和向量数据库 (sqlite-vec) 中。支持最大 4000 字符。',
            style: TextStyle(color: Colors.grey),
          ),
          const SizedBox(height: 16),
          Expanded(
            child: Stack(
              children: [
                TextField(
                  controller: _textController,
                  maxLines: null,
                  maxLength: 4000,
                  expands: true,
                  textAlignVertical: TextAlignVertical.top,
                  decoration: InputDecoration(
                    hintText: _isListening ? '正在聆听，请说话...' : '请输入文本内容...',
                    border: const OutlineInputBorder(),
                    contentPadding: const EdgeInsets.only(
                      left: 12,
                      top: 12,
                      right: 12,
                      bottom: 60, // Leave space for the floating button
                    ),
                  ),
                ),
                Positioned(
                  bottom: 24,
                  right: 12,
                  child: FloatingActionButton.small(
                    onPressed: _speechEnabled ? _toggleListening : null,
                    tooltip: _isListening ? '停止语音输入' : '开始语音输入',
                    backgroundColor: _isListening ? Colors.red : null,
                    child: Icon(
                      _isListening ? Icons.mic : Icons.mic_none,
                      color: _isListening ? Colors.white : null,
                    ),
                  ),
                ),
              ],
            ),
          ),
          if (_isListening)
            const Padding(
              padding: EdgeInsets.only(top: 8.0),
              child: Text(
                '正在聆听...',
                style: TextStyle(color: Colors.red, fontSize: 12),
                textAlign: TextAlign.right,
              ),
            ),
          const SizedBox(height: 16),
          ElevatedButton.icon(
            onPressed: _isSaving ? null : _saveText,
            icon: _isSaving
                ? const SizedBox(
                    width: 20,
                    height: 20,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : const Icon(Icons.save),
            label: Text(_isSaving ? '正在生成 Embedding 并保存...' : '索引并保存文本'),
            style: ElevatedButton.styleFrom(
              padding: const EdgeInsets.symmetric(vertical: 16),
            ),
          ),
        ],
      ),
    );
  }
}
