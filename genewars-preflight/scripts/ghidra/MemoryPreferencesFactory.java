// Original launcher adapter: isolate java.util.prefs from the Windows registry.
// Loader options are deliberately defaults per process; no global preferences read/write.
import java.util.*;
import java.util.prefs.*;
public final class MemoryPreferencesFactory implements PreferencesFactory {
  private static final Node USER=new Node(null,"");
  private static final Node SYSTEM=new Node(null,"");
  public Preferences userRoot(){return USER;}
  public Preferences systemRoot(){return SYSTEM;}
  private static final class Node extends AbstractPreferences {
    final Map<String,String> values=new HashMap<>();
    final Map<String,Node> children=new HashMap<>();
    Node(AbstractPreferences parent,String name){super(parent,name);}
    protected void putSpi(String k,String v){values.put(k,v);}
    protected String getSpi(String k){return values.get(k);}
    protected void removeSpi(String k){values.remove(k);}
    protected void removeNodeSpi(){values.clear();children.clear();}
    protected String[] keysSpi(){return values.keySet().toArray(new String[0]);}
    protected String[] childrenNamesSpi(){return children.keySet().toArray(new String[0]);}
    protected AbstractPreferences childSpi(String n){return children.computeIfAbsent(n,k->new Node(this,k));}
    protected void syncSpi(){}
    protected void flushSpi(){}
  }
}
