import 'dart:io';
import 'package:flutter/material.dart';
import 'package:path_provider/path_provider.dart';
import 'package:nano_vector_app/src/rust/api/simple.dart';
import 'package:nano_vector_app/src/rust/frb_generated.dart';
import 'package:nano_vector_app/index_page.dart';
import 'package:nano_vector_app/search_page.dart';

late AppCore appCore;

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

  // Initialize AppCore with a local model path to avoid downloading on simulator
  appCore = await AppCore.newInstance(
    dbPath: dbPath,
    vectorDbPath: vectorDbPath,
    modelPath: '/Users/finchking/.cache/huggingface/hub/models--BAAI--bge-small-zh-v1.5/snapshots/7999e1d3359715c523056ef9478215996d62a620',
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
