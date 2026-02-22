import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:path_provider/path_provider.dart';
import 'package:nano_vector_app/src/rust/api/simple.dart';
import 'package:nano_vector_app/src/rust/frb_generated.dart';
import 'package:nano_vector_app/index_page.dart';
import 'package:nano_vector_app/search_page.dart';

late AppCore appCore;

/// Extract bundled model files from assets to documents directory on first launch
Future<String> _ensureModelFiles(String baseDir) async {
  final modelDir = '$baseDir/model';
  final modelDirObj = Directory(modelDir);

  // Check if model files already exist
  final configFile = File('$modelDir/config.json');
  final tokenizerFile = File('$modelDir/tokenizer.json');
  final modelFile = File('$modelDir/model.safetensors');

  if (await configFile.exists() &&
      await tokenizerFile.exists() &&
      await modelFile.exists()) {
    debugPrint('Model files already exist at: $modelDir');
    return modelDir;
  }

  // Create model directory
  await modelDirObj.create(recursive: true);
  debugPrint('Extracting model files to: $modelDir');

  // Copy model files from assets
  final files = ['config.json', 'tokenizer.json', 'model.safetensors'];
  for (final fileName in files) {
    debugPrint('Copying $fileName...');
    final data = await rootBundle.load('assets/model/$fileName');
    final file = File('$modelDir/$fileName');
    await file.writeAsBytes(data.buffer.asUint8List());
  }

  debugPrint('Model files extracted successfully');
  return modelDir;
}

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();

  // Initialize paths in app's local directory
  final docDir = await getApplicationDocumentsDirectory();
  final dir = docDir.path;
  final dbPath = '$dir/data/app.db';
  final vectorDbPath = '$dir/data/vector_db';

  // Ensure data dir exists
  Directory('$dir/data').createSync(recursive: true);

  // Extract bundled model files on first launch
  final modelPath = await _ensureModelFiles(dir);

  appCore = await AppCore.newInstance(
    dbPath: dbPath,
    vectorDbPath: vectorDbPath,
    modelPath: modelPath,
  );

  runApp(const NanoVectorApp());
}

class NanoVectorApp extends StatelessWidget {
  const NanoVectorApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: '文本检索应用',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
      home: const MainScreen(),
    );
  }
}

class MainScreen extends StatefulWidget {
  const MainScreen({super.key});

  @override
  State<MainScreen> createState() => _MainScreenState();
}

class _MainScreenState extends State<MainScreen> {
  int _selectedIndex = 0;

  static const List<Widget> _pages = <Widget>[
    IndexPage(),
    SearchPage(),
  ];

  void _onItemTapped(int index) {
    setState(() {
      _selectedIndex = index;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('文本检索助手'),
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
      ),
      body: IndexedStack(
        index: _selectedIndex,
        children: _pages,
      ),
      bottomNavigationBar: BottomNavigationBar(
        items: const <BottomNavigationBarItem>[
          BottomNavigationBarItem(
            icon: Icon(Icons.note_add),
            label: '索引文本',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.manage_search),
            label: '语义检索',
          ),
        ],
        currentIndex: _selectedIndex,
        onTap: _onItemTapped,
      ),
    );
  }
}
