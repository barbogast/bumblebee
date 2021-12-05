import { useState } from 'react';
import { Layout, Menu } from 'antd';
import './App.css';
import CompareScreen from './CompareScreen';

const { Header, Content, Footer, Sider } = Layout;

function App() {
  const [collapsed, setCollapsed] = useState(false);

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Sider collapsible collapsed={collapsed} onCollapse={() => setCollapsed(!collapsed)}>
        <div className='logo' />
        <Menu
          onClick={({ key }) => console.log(key)}
          theme='dark'
          defaultSelectedKeys={['compare']}
          mode='inline'
        >
          <Menu.Item key='compare'>Compare Directories</Menu.Item>
          <Menu.Item key='disk-space'>Analyze disk space</Menu.Item>
        </Menu>
      </Sider>
      <Layout className='site-layout'>
        <Header className='site-layout-background' style={{ padding: 0 }} />
        <Content style={{ margin: '0 16px' }}>
          <div className='site-layout-background' style={{ padding: 24, minHeight: 360 }}>
            <CompareScreen />
          </div>
        </Content>
        <Footer style={{ textAlign: 'center' }}>XAnt Design ©2018 Created by Ant UED</Footer>
      </Layout>
    </Layout>
  );
}

export default App;
